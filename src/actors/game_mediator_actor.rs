use actix::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Sender;
use uuid::Uuid;

use crate::actors::{message_types::*, WebsocketActor};
use crate::protocol::{GameStateUpdate, PlayerAction};

/// Actor that facilitates communication between the websocket actors and the game engine
pub struct GameMediatorActor {
  game_running: bool,
  registered: HashSet<Uuid>,
  actors: HashMap<Uuid, Addr<WebsocketActor>>,
  send_start_game: Sender<Vec<Uuid>>,
}

impl GameMediatorActor {
  pub fn new(send_start_game: Sender<Vec<Uuid>>) -> Self {
    Self {
      game_running: false,
      registered: HashSet::new(),
      actors: HashMap::new(),
      send_start_game,
    }
  }
}

impl Actor for GameMediatorActor {
  type Context = Context<Self>;
}

impl Handler<IsGameRunning> for GameMediatorActor {
  type Result = bool;

  fn handle(&mut self, _: IsGameRunning, _: &mut Self::Context) -> Self::Result {
    self.game_running
  }
}

impl Handler<IsPlayerRegistered> for GameMediatorActor {
  type Result = bool;

  fn handle(&mut self, IsPlayerRegistered(player_id): IsPlayerRegistered, _: &mut Self::Context) -> Self::Result {
    self.registered.contains(&player_id)
  }
}

impl Handler<IsPlayerConnected> for GameMediatorActor {
  type Result = bool;

  fn handle(&mut self, IsPlayerConnected(player_id): IsPlayerConnected, _: &mut Self::Context) -> Self::Result {
    self.actors.contains_key(&player_id)
  }
}

impl Handler<Connect> for GameMediatorActor {
  type Result = ConnectResponse;

  fn handle(&mut self, Connect(player_id, recipient): Connect, _: &mut Self::Context) -> Self::Result {
    if self.actors.contains_key(&player_id) {
      return ConnectResponse::AlreadyConnected;
    }

    if self.game_running && !self.registered.contains(&player_id) {
      return ConnectResponse::NotRegistered;
    }

    self.actors.insert(player_id, recipient);

    ConnectResponse::Ok
  }
}

impl Handler<Disconnect> for GameMediatorActor {
  type Result = ();

  fn handle(&mut self, Disconnect(player_id): Disconnect, _: &mut Self::Context) -> Self::Result {
    // Force remove the entry, even if it doesn't exist
    self.actors.remove(&player_id);
  }
}

impl Handler<GameStateUpdate> for GameMediatorActor {
  type Result = ();

  fn handle(&mut self, update: GameStateUpdate, _: &mut Self::Context) -> Self::Result {}
}

impl Handler<GameEngineCrash> for GameMediatorActor {
  type Result = ();

  fn handle(&mut self, err: GameEngineCrash, _: &mut Self::Context) -> Self::Result {}
}
