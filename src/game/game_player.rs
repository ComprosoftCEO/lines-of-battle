use actix::prelude::*;
use rlua::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::sync::{mpsc::Receiver, Arc, Mutex};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

use crate::actors::{shared_messages::*, GameMediatorActor};
use crate::errors::GameEngineError;
use crate::protocol::{game::GameState, PlayerAction};

const SECONDS_PER_GAME: u32 = 60 * 3;
const MAX_TRIES: usize = 5;

/// Encapsulates the logic of running the Lua game engine on a given thread
pub struct GamePlayer {
  lua: Lua,
  recv_start_game: Receiver<Vec<Uuid>>,
  recv_player_actions: Receiver<(Uuid, PlayerAction)>,
  mediator_addr: Addr<GameMediatorActor>,

  player_order: Arc<Vec<Uuid>>,
  players_remaining: Arc<Mutex<HashSet<Uuid>>>,
  seconds_per_game: u32,
  seconds_left: u32,
}

#[derive(Clone)]
struct GamePlayerUserData {
  mediator_addr: Addr<GameMediatorActor>,
  player_order: Arc<Vec<Uuid>>,
  players_remaining: Arc<Mutex<HashSet<Uuid>>>,
  seconds_per_game: u32,
  seconds_left: u32,
}

impl GamePlayer {
  /// Construct a new game player object
  ///   This validates the lua code when it is loaded
  pub fn new(
    lua_file: impl AsRef<Path>,
    recv_start_game: Receiver<Vec<Uuid>>,
    recv_player_actions: Receiver<(Uuid, PlayerAction)>,
    mediator_addr: Addr<GameMediatorActor>,
  ) -> Result<Self, GameEngineError> {
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
      recv_start_game,
      recv_player_actions,
      mediator_addr,
      player_order: Arc::default(),
      players_remaining: Arc::default(),
      seconds_per_game: SECONDS_PER_GAME,
      seconds_left: 0,
    })
  }

  /// Get the user data from the game state
  ///   This is passed to Lua as a "context" variable
  fn get_user_data(&self) -> GamePlayerUserData {
    GamePlayerUserData {
      mediator_addr: self.mediator_addr.clone(),
      player_order: self.player_order.clone(),
      players_remaining: self.players_remaining.clone(),
      seconds_per_game: self.seconds_per_game,
      seconds_left: self.seconds_left,
    }
  }

  /// Run the game engine
  pub fn run_game(&mut self) {
    if let Err(e) = self.run_internal() {
      log::error!("Fatal error: {}", e.get_developer_notes());
      self.mediator_addr.do_send(GameEngineCrash);
    }
  }

  ///
  /// Run the game and return a GameEngineError on a fatal error
  ///
  fn run_internal(&mut self) -> Result<(), GameEngineError> {
    loop {
      // Wait for the mediator to say the game is ready to start
      let player_order = self
        .recv_start_game
        .recv()
        .map_err(|e| GameEngineError::ChannelClosed("start_game", e))?;

      // Initialize the game!
      let initial_state = Self::trap_errors(MAX_TRIES, || self.init_game(&player_order))?;
      self.mediator_addr.do_send(Init::new(initial_state, self.seconds_left));

      // Run until there is no time left
      while self.seconds_left > 0 {
        // Sleep for roughtly one second before running the next tick
        thread::sleep(Duration::from_secs(1));
        self.seconds_left -= 1;

        // Read the list of player actions from the channel
        //  Filter any actions for players that have died (just to be extra safe)
        let players_remaining = self.players_remaining.lock().unwrap();
        let player_actions: HashMap<_, _> = self
          .recv_player_actions
          .try_iter()
          .filter(|(id, _)| players_remaining.contains(id))
          .collect();
        drop(players_remaining);

        // Update the game state
        let next_state = Self::trap_errors(MAX_TRIES, || self.tick_game(&player_actions))?;

        // Notify the mediator of the change
        if self.seconds_left > 0 {
          self
            .mediator_addr
            .do_send(NextState::new(next_state, player_actions, self.seconds_left));
        } else {
          self.mediator_addr.do_send(GameEnded::new(
            self.players_remaining.lock().unwrap().clone(),
            next_state,
            player_actions,
          ));
        }
      }
    }
  }

  /// Handle game initialization with the given player order
  fn init_game(&mut self, player_order: &Vec<Uuid>) -> Result<GameState, GameEngineError> {
    // Initialize game player variables
    self.player_order = Arc::new(player_order.clone());
    self.seconds_left = self.seconds_per_game;
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
impl LuaUserData for GamePlayerUserData {
  fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_method("notifyPlayerKilled", |_, this, uuid: String| {
      let player_id: Uuid = Uuid::from_str(&uuid).map_err(|_| LuaError::RuntimeError("Invalid UUID".into()))?;

      // Update the internal list of players remaining
      this.players_remaining.lock().unwrap().remove(&player_id);

      // Also notify the mediator
      this.mediator_addr.do_send(PlayerKilled::new(player_id));

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

    methods.add_method("getSecondsLeft", |_, this, _: ()| {
      Ok((this.seconds_left, this.seconds_per_game))
    });
  }
}
