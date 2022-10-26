use dotenv::dotenv;
use log::LevelFilter;
use rand::seq::SliceRandom;
use rlua::prelude::*;
use simple_logger::SimpleLogger;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;
use uuid::Uuid;

use game_server::config::{self, DEFAULT_LUA_FILE};
use game_server::errors::GameEngineError;
use game_server::protocol::{actions::*, game::GameState, PlayerAction, TaggedRequest};

const MAX_TRIES: usize = 5;

/// Simple test of the game engine code to detect runtime bugs
#[derive(StructOpt)]
struct Opt {
  /// Lua file containing the game engine code
  #[structopt(long, env, default_value = DEFAULT_LUA_FILE)]
  lua_file: String,

  /// Number of total "ticks" for a complete round in the game
  #[structopt(long, env, default_value = "180")]
  ticks_per_game: u32,

  /// Number of players in the game
  #[structopt(long, default_value = "4")]
  num_players: usize,

  /// If set, also shows the debug output
  #[structopt(short = "d", long)]
  show_debug: bool,
}

impl Opt {
  /// Update the environment variables with the command-line options
  pub fn update_environment(&self) {
    env::set_var("LUA_FILE", &self.lua_file);
    env::set_var("TICKS_PER_GAME", self.ticks_per_game.to_string());
  }

  pub fn should_show_debug(&self) -> bool {
    self.show_debug
  }

  pub fn get_num_players(&self) -> usize {
    self.num_players
  }
}

//
// Main program entry point
//
fn main() -> anyhow::Result<()> {
  // Load our ".env" configuration file
  dotenv().ok();

  // Parse command-line arguments
  let opt: Opt = Opt::from_args();
  opt.update_environment();

  // Configure the logger system
  SimpleLogger::new().init()?;
  if opt.should_show_debug() {
    log::info!("Turning on debug output");
    log::set_max_level(LevelFilter::Debug);
  } else {
    log::set_max_level(LevelFilter::Info);
  }

  // Load and run the game
  let mut game_player = TestGamePlayer::new(config::get_lua_file(), opt.get_num_players())?;
  game_player.run_game()?;

  Ok(())
}

/// Encapsulates the logic of running the Lua game engine on a given thread
pub struct TestGamePlayer {
  lua: Lua,
  num_players: usize,
  player_order: Arc<Vec<Uuid>>,
  players_remaining: Arc<Mutex<HashSet<Uuid>>>,
  ticks_per_game: u32,
  ticks_left: u32,
}

#[derive(Clone)]
struct TestGamePlayerUserData {
  player_order: Arc<Vec<Uuid>>,
  players_remaining: Arc<Mutex<HashSet<Uuid>>>,
  ticks_per_game: u32,
  ticks_left: u32,
}

impl TestGamePlayer {
  /// Construct a new test game player object
  ///   This validates the lua code when it is loaded
  pub fn new(lua_file: impl AsRef<Path>, num_players: usize) -> Result<Self, GameEngineError> {
    // Read and execute the Lua code
    let lua_code = fs::read_to_string(&lua_file).map_err(GameEngineError::FailedToReadLuaFile)?;

    let lua = Lua::new();
    lua.context::<_, Result<(), GameEngineError>>(|ctx| {
      // Add the parent directory (if it exists) to the Lua path
      //  Silently fail on errors
      if let Some(parent_dir) = lua_file.as_ref().parent() {
        if let Some(parent_dir) = parent_dir.join("?.lua").to_str() {
          log::debug!("Adding directory '{}' to Lua path", parent_dir);
          if let Err(e) = ctx
            .load(&format!(r#"package.path = [[{};]] .. package.path"#, parent_dir))
            .exec()
          {
            log::warn!("Failed to update the Lua path: {}", e);
          }
        }
      }

      // Run the file
      ctx
        .load(&lua_code)
        .exec()
        .map_err(GameEngineError::FailedToRunLuaFile)?;

      // Make sure required methods exist
      let globals = ctx.globals();
      globals
        .get::<_, LuaFunction>("Init")
        .map_err(|e| GameEngineError::MissingRequiredLuaMethod("Init", e))?;

      globals
        .get::<_, LuaFunction>("Update")
        .map_err(|e| GameEngineError::MissingRequiredLuaMethod("Update", e))?;

      Ok(())
    })?;

    Ok(Self {
      lua,
      num_players,
      player_order: Arc::default(),
      players_remaining: Arc::default(),
      ticks_per_game: config::get_ticks_per_game(),
      ticks_left: 0,
    })
  }

  /// Get the user data from the game state
  ///   This is passed to Lua as a "context" variable
  fn get_user_data(&self) -> TestGamePlayerUserData {
    TestGamePlayerUserData {
      player_order: self.player_order.clone(),
      players_remaining: self.players_remaining.clone(),
      ticks_per_game: self.ticks_per_game,
      ticks_left: self.ticks_left,
    }
  }

  /// Run the test game engine
  pub fn run_game(&mut self) -> Result<(), GameEngineError> {
    if let Err(e) = self.run_internal() {
      log::error!("Fatal error: {}", e.get_developer_notes());
      Err(e)
    } else {
      Ok(())
    }
  }

  /// A game round is running if:
  ///
  /// 1. There are 2 or more players left in the game
  /// 2. AND there is time left on the clock
  fn is_round_running(&self) -> bool {
    self.ticks_left > 0 && self.players_remaining.lock().unwrap().len() > 1
  }

  ///
  /// Run the game and return a GameEngineError on a fatal error
  ///
  fn run_internal(&mut self) -> Result<(), GameEngineError> {
    log::info!("Generating random list of players");

    // Wait for the mediator to say the game is ready to start
    let player_order = (0..self.num_players).into_iter().map(|_| Uuid::new_v4()).collect();

    // Initialize the game!
    log::info!("Initializing game engine...");

    let initial_state = Self::trap_errors(MAX_TRIES, || self.init_game(&player_order))?;
    log::debug!(
      "Initial state: {}",
      serde_json::to_string_pretty(&initial_state).unwrap()
    );

    // Run until there is no time left
    while self.is_round_running() {
      self.ticks_left -= 1;
      log::info!(
        "Game engine running - {} tick{} remaining",
        self.ticks_left,
        if self.ticks_left == 1 { "" } else { "s" }
      );

      // Pick random actions for the players
      //  Filter any actions for players that have died (just to be extra safe)
      let players_remaining = self.players_remaining.lock().unwrap();
      let player_actions: HashMap<_, _> = self
        .pick_random_player_actions()
        .into_iter()
        .filter(|(id, _)| players_remaining.contains(id))
        .collect();
      drop(players_remaining);

      // Update the game state
      let next_state = Self::trap_errors(MAX_TRIES, || self.tick_game(&player_actions))?;
      log::debug!("Next state: {}", serde_json::to_string_pretty(&next_state).unwrap());
    }

    log::info!("Game ended without any problems");

    // Show the winner(s)
    let players_remaining = self.players_remaining.lock().unwrap();
    if players_remaining.len() > 1 {
      log::info!("Winners: {:#?}", players_remaining);
    } else {
      log::info!("Winner: {:#?}", players_remaining);
    }

    Ok(())
  }

  ///
  /// Handle game initialization with the given player order
  ///
  fn init_game(&mut self, player_order: &Vec<Uuid>) -> Result<GameState, GameEngineError> {
    // Initialize game player variables
    self.player_order = Arc::new(player_order.clone());
    self.ticks_left = self.ticks_per_game;
    self.players_remaining = Arc::new(Mutex::new(player_order.iter().cloned().collect()));

    // Run the Lua Init() method and return the initial game state as JSON
    self.lua.context::<_, Result<_, GameEngineError>>(|ctx| {
      let init = ctx
        .globals()
        .get::<_, LuaFunction>("Init")
        .map_err(|e| GameEngineError::MissingRequiredLuaMethod("Init", e))?;

      let user_data = self.get_user_data();
      let player_order: Vec<_> = self.player_order.iter().map(Uuid::to_string).collect();

      let lua_game_state = init
        .call::<_, LuaValue>((user_data, player_order))
        .map_err(|e| GameEngineError::FailedToRunMethod("Init", e))?;

      let json_game_state: GameState = rlua_serde::from_value(lua_game_state).map_err(GameEngineError::LuaToJSON)?;

      Ok(json_game_state)
    })
  }

  /// Randomly pick an action (and random direction if necessary) for each player
  fn pick_random_player_actions(&self) -> HashMap<Uuid, PlayerAction> {
    let mut rng = rand::thread_rng();

    self
      .player_order
      .iter()
      .map(|id| {
        let direction = *[Direction::Up, Direction::Down, Direction::Left, Direction::Right]
          .choose(&mut rng)
          .unwrap();

        let action = [
          (TaggedRequest::new(PlayerActionEnum::Move(MoveAction { direction })), 5),
          (
            TaggedRequest::new(PlayerActionEnum::Attack(AttackAction { direction })),
            5,
          ),
          (TaggedRequest::new(PlayerActionEnum::DropWeapon), 2),
        ]
        .choose_weighted(&mut rng, |(_, w)| *w)
        .unwrap()
        .clone()
        .0;

        (*id, action)
      })
      .collect()
  }

  /// Perform a single game tick:
  ///   Call the Lua Update() method and return the next game state
  ///
  /// Does NOT handle the logic for "seconds left"
  fn tick_game(&mut self, player_actions: &HashMap<Uuid, PlayerAction>) -> Result<GameState, GameEngineError> {
    self.lua.context(|ctx| {
      let player_actions: HashMap<String, LuaValue> = player_actions
        .iter()
        .map(|(id, action)| {
          let id = id.to_string();
          let value = rlua_serde::to_value(ctx, action).map_err(GameEngineError::JSONToLua)?;
          Ok((id, value))
        })
        .collect::<Result<_, _>>()?;

      let update = ctx
        .globals()
        .get::<_, LuaFunction>("Update")
        .map_err(|e| GameEngineError::MissingRequiredLuaMethod("Update", e))?;

      let user_data = self.get_user_data();
      let lua_game_state = update
        .call::<_, LuaValue>((user_data, player_actions))
        .map_err(|e| GameEngineError::FailedToRunMethod("Update", e))?;

      let json_game_state: GameState = rlua_serde::from_value(lua_game_state).map_err(GameEngineError::LuaToJSON)?;

      Ok(json_game_state)
    })
  }

  /// Helper function to retry a given number of times before throwing an error
  fn trap_errors<F, R>(max_tries: usize, mut func: F) -> Result<R, GameEngineError>
  where
    F: FnMut() -> Result<R, GameEngineError>,
  {
    let mut tries = 0;
    loop {
      match func() {
        Ok(r) => return Ok(r),
        Err(e) => {
          tries = tries + 1;
          log::error!(
            "Game engine error: {} (Attempt {} / {})",
            e.get_developer_notes(),
            tries,
            max_tries
          );

          if tries >= max_tries {
            return Err(e);
          }
        },
      }
    }
  }
}

//
// Helper context methods that get passed into Lua
//
impl LuaUserData for TestGamePlayerUserData {
  fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_method("notifyPlayerKilled", |_, this, uuid: String| {
      let player_id: Uuid = Uuid::from_str(&uuid).map_err(|_| LuaError::RuntimeError("Invalid UUID".into()))?;

      // Update the internal list of players remaining
      this.players_remaining.lock().unwrap().remove(&player_id);
      log::info!("Player {} killed", player_id);

      Ok(())
    });

    methods.add_method("getPlayerOrder", |_, this, _: ()| {
      Ok(this.player_order.iter().map(Uuid::to_string).collect::<Vec<_>>())
    });

    methods.add_method("getPlayersRemaining", |_, this, _: ()| {
      Ok(
        this
          .players_remaining
          .lock()
          .unwrap()
          .iter()
          .map(|id| (id.to_string(), true))
          .collect::<HashMap<_, _>>(),
      )
    });

    methods.add_method("getTicksLeft", |_, this, _: ()| {
      Ok((this.ticks_left, this.ticks_per_game))
    });
  }
}
