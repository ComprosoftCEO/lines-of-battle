use actix::prelude::*;
use bytestring::ByteString;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::jwt::JWTPlayerData;
use crate::protocol::*;

/// Fatal error has caused the game engine to crash - Server must reboot!
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct GameEngineCrash;

///
/// Broadcast update about registration
///
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct RegistrationUpdate(ByteString);

impl RegistrationUpdate {
  pub fn waiting_on_players(
    players: HashMap<Uuid, JWTPlayerData>,
    min_players_needed: usize,
    max_players_allowed: usize,
  ) -> Self {
    Self(
      RegistrationUpdateEnum::WaitingOnPlayers {
        players,
        min_players_needed,
        max_players_allowed,
      }
      .into_bytestring(),
    )
  }

  pub fn game_starting_soon(
    players: HashMap<Uuid, JWTPlayerData>,
    min_players_needed: usize,
    max_players_allowed: usize,
    seconds_left: u32,
  ) -> Self {
    Self(
      RegistrationUpdateEnum::GameStartingSoon {
        players,
        min_players_needed,
        max_players_allowed,
        seconds_left,
      }
      .into_bytestring(),
    )
  }

  pub fn game_starting(players: HashMap<Uuid, JWTPlayerData>, player_order: Vec<Uuid>) -> Self {
    Self(RegistrationUpdateEnum::GameStarting { players, player_order }.into_bytestring())
  }
}

///
/// Game is now being initialized, registration is permenantly closed
///
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct GameStarting(ByteString);

impl GameStarting {
  pub fn new(players: HashMap<Uuid, JWTPlayerData>, player_order: Vec<Uuid>) -> Self {
    Self(RegistrationUpdateEnum::GameStarting { players, player_order }.into_bytestring())
  }
}

///
/// Broadcast the init message with the first game state
///
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct Init(ByteString);

impl Init {
  pub fn new(game_state: GameState, ticks_left: u32, seconds_per_tick: u32) -> Self {
    Self(
      GameStateUpdate::Init {
        game_state,
        ticks_left,
        seconds_per_tick,
      }
      .into_bytestring(),
    )
  }
}

///
/// Broadcast the next state message
///
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct NextState(ByteString);

impl NextState {
  pub fn new(
    game_state: GameState,
    actions_taken: HashMap<Uuid, PlayerAction>,
    ticks_left: u32,
    seconds_per_tick: u32,
  ) -> Self {
    Self(
      GameStateUpdate::NextState {
        game_state,
        actions_taken,
        ticks_left,
        seconds_per_tick,
      }
      .into_bytestring(),
    )
  }
}

/// Broadcast the player killed message
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct PlayerKilled {
  player_id: Uuid,
  data: ByteString,
}

impl PlayerKilled {
  pub fn new(player_id: Uuid) -> Self {
    Self {
      player_id,
      data: GameStateUpdate::PlayerKilled { id: player_id }.into_bytestring(),
    }
  }

  pub fn get_player_id(&self) -> Uuid {
    self.player_id
  }
}

/// Broadcast the game ended message
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct GameEnded(ByteString);

impl GameEnded {
  pub fn new(winners: HashSet<Uuid>, game_state: GameState, actions_taken: HashMap<Uuid, PlayerAction>) -> Self {
    Self(
      GameStateUpdate::GameEnded {
        winners,
        game_state,
        actions_taken,
      }
      .into_bytestring(),
    )
  }
}

impl ToBytestring for RegistrationUpdate {
  fn to_bytestring(&self) -> ByteString {
    self.0.clone()
  }

  fn into_bytestring(self) -> ByteString {
    self.0
  }
}

impl ToBytestring for Init {
  fn to_bytestring(&self) -> ByteString {
    self.0.clone()
  }

  fn into_bytestring(self) -> ByteString {
    self.0
  }
}

impl ToBytestring for GameStarting {
  fn to_bytestring(&self) -> ByteString {
    self.0.clone()
  }

  fn into_bytestring(self) -> ByteString {
    self.0
  }
}

impl ToBytestring for NextState {
  fn to_bytestring(&self) -> ByteString {
    self.0.clone()
  }

  fn into_bytestring(self) -> ByteString {
    self.0
  }
}

impl ToBytestring for PlayerKilled {
  fn to_bytestring(&self) -> ByteString {
    self.data.clone()
  }

  fn into_bytestring(self) -> ByteString {
    self.data
  }
}

impl ToBytestring for GameEnded {
  fn to_bytestring(&self) -> ByteString {
    self.0.clone()
  }

  fn into_bytestring(self) -> ByteString {
    self.0
  }
}
