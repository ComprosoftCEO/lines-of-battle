use actix::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Sender;
use std::time::Duration;
use uuid::Uuid;

use crate::actors::{mediator_messages::*, shared_messages::*, ViewerActor, WebsocketActor};
use crate::config;
use crate::game::ServerState;
use crate::jwt::JWTPlayerData;

/// Actor that facilitates communication between the websocket actors and the game engine
pub struct GameMediatorActor {
  server_state: ServerState,
  registered: HashMap<Uuid, JWTPlayerData>, // Stores ID and other player data
  actors: HashMap<Uuid, Addr<WebsocketActor>>,
  viewers: HashSet<Addr<ViewerActor>>,
  player_order: Option<Vec<Uuid>>,
  send_start_game: Sender<Vec<Uuid>>,
  min_players_required: usize,
  max_players_allowed: usize,
  lobby_wait_secs: u32,
  secs_left: u32,
}

impl GameMediatorActor {
  /// Construct a new game mediator actor with the given channel
  pub fn new(send_start_game: Sender<Vec<Uuid>>) -> Self {
    let min_players_required = config::get_min_players_required();
    let mut max_players_allowed = config::get_max_players_allowed();

    if max_players_allowed < min_players_required {
      log::warn!(
        "MAX_PLAYERS_ALLOWED is smaller than MIN_PLAYERS_REQUIRED ({} < {}), using value '{}' instead",
        max_players_allowed,
        min_players_required,
        min_players_required
      );
      max_players_allowed = min_players_required;
    }

    let lobby_wait_secs = config::get_lobby_wait_time_seconds();

    Self {
      server_state: ServerState::Registration,
      registered: HashMap::new(),
      actors: HashMap::new(),
      viewers: HashSet::new(),
      player_order: None,
      send_start_game,
      min_players_required,
      max_players_allowed,
      lobby_wait_secs,
      secs_left: lobby_wait_secs,
    }
  }

  /// Broadcast a message - Should accept a type that can be easily cloned
  fn broadcast_all<M>(&self, data: M)
  where
    M: Clone + Message + Send + 'static,
    <M as actix::Message>::Result: Send,
    WebsocketActor: Handler<M>,
    ViewerActor: Handler<M>,
  {
    // Send to all actors
    for (_, actor) in self.actors.iter() {
      actor.do_send(data.clone());
    }

    // Also send to all viewers
    for viewer in self.viewers.iter() {
      viewer.do_send(data.clone());
    }
  }

  /// Send an update with the latest registration details
  fn broadcast_registration_update(&self) {
    if self.registered.len() < self.min_players_required {
      self.broadcast_all(RegistrationUpdate::waiting_on_players(
        self.registered.clone(),
        self.min_players_required,
      ));
    } else {
      self.broadcast_all(RegistrationUpdate::game_starting_soon(
        self.registered.clone(),
        self.min_players_required,
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
    if self.server_state != ServerState::Registration {
      return;
    }

    if self.registered.len() < self.min_players_required {
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
    self.player_order = Some(player_order.clone());
    self.server_state = ServerState::Initializing;

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

    if !self.server_state.can_change_registration() && !self.registered.contains_key(&player_id) {
      return ConnectResponse::NotRegistered;
    }

    self.actors.insert(player_id, addr);

    ConnectResponse::Ok(self.server_state)
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

impl Handler<ConnectViewer> for GameMediatorActor {
  type Result = ConnectViewerResponse;

  fn handle(&mut self, ConnectViewer(addr): ConnectViewer, _: &mut Self::Context) -> Self::Result {
    self.viewers.insert(addr);
    ConnectViewerResponse(self.server_state)
  }
}

impl Handler<DisconnectViewer> for GameMediatorActor {
  type Result = ();

  fn handle(&mut self, DisconnectViewer(addr): DisconnectViewer, _: &mut Self::Context) -> Self::Result {
    self.viewers.remove(&addr);
  }
}

impl Handler<Register> for GameMediatorActor {
  type Result = RegisterResponse;

  fn handle(&mut self, Register { id, data }: Register, _: &mut Self::Context) -> Self::Result {
    if !self.server_state.can_change_registration() {
      return RegisterResponse::GameAlreadyStarted;
    }

    let not_enough_before = self.registered.len() < self.min_players_required;

    // Since this is idempotent, register the player ONLY if they aren't already registered
    if !self.registered.contains_key(&id) {
      // Make sure we aren't at the maximum players yet
      if self.registered.len() >= self.max_players_allowed {
        return RegisterResponse::TooManyRegistered {
          max_allowed: self.max_players_allowed,
        };
      }

      self.registered.insert(id, data);
    }

    // Reset the lobby counter when the count just goes over the minimum number of players needed
    if not_enough_before && self.registered.len() >= self.min_players_required {
      self.secs_left = self.lobby_wait_secs;
    }

    // Broadcast the update
    self.broadcast_registration_update();

    RegisterResponse::Success
  }
}

impl Handler<Unregister> for GameMediatorActor {
  type Result = bool;

  fn handle(&mut self, Unregister { id }: Unregister, _: &mut Self::Context) -> Self::Result {
    if !self.server_state.can_change_registration() {
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
    self.server_state = ServerState::Running;
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
    self.player_order = None;
    self.server_state = ServerState::Registration;
    self.broadcast_all(game_ended);
  }
}

impl Handler<GameEngineCrash> for GameMediatorActor {
  type Result = ();

  fn handle(&mut self, _: GameEngineCrash, _: &mut Self::Context) -> Self::Result {
    self.server_state = ServerState::FatalError;
    self.player_order = None;

    for (_, actor) in self.actors.iter() {
      actor.do_send(GameEngineCrash);
    }
  }
}

impl Handler<GetRegisteredPlayers> for GameMediatorActor {
  type Result = GetRegisteredPlayersResponse;

  fn handle(&mut self, _: GetRegisteredPlayers, _: &mut Self::Context) -> Self::Result {
    GetRegisteredPlayersResponse {
      players: self.registered.clone(),
      player_order: self.player_order.clone(),
    }
  }
}
