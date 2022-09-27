use actix::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::time::Duration;
use uuid::Uuid;

use crate::actors::{mediator_messages::*, shared_messages::*, WebsocketActor};
use crate::game::GameState;
use crate::jwt::JWTPlayerData;

const MIN_PLAYERS_NEEDED: usize = 2;
const LOBBY_WAIT_SECS: u32 = 10;

/// Actor that facilitates communication between the websocket actors and the game engine
pub struct GameMediatorActor {
  game_state: GameState,
  registered: HashMap<Uuid, JWTPlayerData>, // Stores ID and other player data
  actors: HashMap<Uuid, Addr<WebsocketActor>>,
  send_start_game: Sender<Vec<Uuid>>,
  min_players_needed: usize,
  secs_left: u32,
}

impl GameMediatorActor {
  /// Construct a new game mediator actor with the given channel
  pub fn new(send_start_game: Sender<Vec<Uuid>>) -> Self {
    Self {
      game_state: GameState::Registration,
      registered: HashMap::new(),
      actors: HashMap::new(),
      send_start_game,
      min_players_needed: MIN_PLAYERS_NEEDED,
      secs_left: LOBBY_WAIT_SECS,
    }
  }

  /// Broadcast a message - Should accept a type that can be easily cloned
  fn broadcast_all<M>(&self, data: M)
  where
    M: Clone + Message + Send + 'static,
    <M as actix::Message>::Result: Send,
    WebsocketActor: Handler<M>,
  {
    for (_, actor) in self.actors.iter() {
      actor.do_send(data.clone());
    }
  }

  /// Send an update with the latest registration details
  fn broadcast_registration_update(&self) {
    if self.registered.len() < self.min_players_needed {
      self.broadcast_all(RegistrationUpdate::waiting_on_players(
        self.registered.clone(),
        self.min_players_needed,
      ));
    } else {
      self.broadcast_all(RegistrationUpdate::game_starting_soon(
        self.registered.clone(),
        self.min_players_needed,
        self.secs_left,
      ));
    };
  }
}

///
/// Make GameMediatorActor into an actor that can run in the background
///
impl Actor for GameMediatorActor {
  type Context = Context<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    ctx.run_interval(Duration::from_secs(1), |this, _ctx| this.tick_registration_update());
  }
}

//
// Handle registration "tick" logic
//
impl GameMediatorActor {
  /// Run once every second to update the registration state
  fn tick_registration_update(&mut self) {
    if self.game_state != GameState::Registration {
      return;
    }

    if self.registered.len() < self.min_players_needed {
      return;
    }

    // Count down the number of seconds
    self.secs_left -= 1;
    if self.secs_left == 0 {
      // Lobby time is up! Start the game now!
      return self.start_game();
    }

    // Send an update that the game is starting soon...
    self.broadcast_registration_update();
  }

  fn start_game(&mut self) {
    // Pick a random order for the players
    let player_order: Vec<_> = self.registered.iter().map(|(id, _)| id.clone()).collect();
    self.game_state = GameState::Initializing;

    // Notify all players that game is starting
    self.broadcast_all(GameStarting::new(self.registered.clone(), player_order.clone()));

    // Send the message for the game engine to start
    self.send_start_game.send(player_order).ok();
  }
}

impl Handler<Connect> for GameMediatorActor {
  type Result = ConnectResponse;

  fn handle(&mut self, Connect(player_id, addr): Connect, _: &mut Self::Context) -> Self::Result {
    if self.actors.contains_key(&player_id) {
      return ConnectResponse::AlreadyConnected;
    }

    if !self.game_state.can_change_registration() && !self.registered.contains_key(&player_id) {
      return ConnectResponse::NotRegistered;
    }

    self.actors.insert(player_id, addr);

    ConnectResponse::Ok(self.game_state)
  }
}

impl Handler<Disconnect> for GameMediatorActor {
  type Result = ();

  fn handle(&mut self, Disconnect(player_id, player_addr): Disconnect, _: &mut Self::Context) -> Self::Result {
    if let Some(addr) = self.actors.get(&player_id) {
      if addr == &player_addr {
        self.actors.remove(&player_id);
      }
    }
  }
}

impl Handler<Register> for GameMediatorActor {
  type Result = bool;

  fn handle(&mut self, Register { id, data }: Register, _: &mut Self::Context) -> Self::Result {
    if !self.game_state.can_change_registration() {
      return false;
    }

    let not_enough_before = self.registered.len() < self.min_players_needed;

    // Force register the player, even if they are already registered
    self.registered.insert(id, data);

    // Reset the lobby counter if needed
    if not_enough_before && self.registered.len() >= self.min_players_needed {
      self.secs_left = LOBBY_WAIT_SECS;
    }

    // Broadcast the update
    self.broadcast_registration_update();

    true
  }
}

impl Handler<Unregister> for GameMediatorActor {
  type Result = bool;

  fn handle(&mut self, Unregister { id }: Unregister, _: &mut Self::Context) -> Self::Result {
    if !self.game_state.can_change_registration() {
      return false;
    }

    // Force unregister the player, even if they are already unregistered
    self.registered.remove(&id);

    // Broadcast the update
    self.broadcast_registration_update();

    true
  }
}

impl Handler<Init> for GameMediatorActor {
  type Result = ();

  fn handle(&mut self, init: Init, _: &mut Self::Context) -> Self::Result {
    self.game_state = GameState::Running;
    self.broadcast_all(init);
  }
}

impl Handler<NextState> for GameMediatorActor {
  type Result = ();

  fn handle(&mut self, next_state: NextState, _: &mut Self::Context) -> Self::Result {
    self.broadcast_all(next_state);
  }
}

impl Handler<PlayerKilled> for GameMediatorActor {
  type Result = ();

  fn handle(&mut self, player_killed: PlayerKilled, _: &mut Self::Context) -> Self::Result {
    self.broadcast_all(player_killed);
  }
}

impl Handler<GameEnded> for GameMediatorActor {
  type Result = ();

  fn handle(&mut self, game_ended: GameEnded, _: &mut Self::Context) -> Self::Result {
    self.registered.clear();
    self.game_state = GameState::Registration;
    self.broadcast_all(game_ended);
  }
}

impl Handler<GameEngineCrash> for GameMediatorActor {
  type Result = ();

  fn handle(&mut self, _: GameEngineCrash, _: &mut Self::Context) -> Self::Result {
    self.game_state = GameState::FatalError;
    for (_, actor) in self.actors.iter() {
      actor.do_send(GameEngineCrash);
    }
  }
}
