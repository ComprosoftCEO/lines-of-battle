use actix::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

use crate::jwt::JWTPlayerData;

/// Notify the mediator that the game state has been updated
#[derive(Debug, Clone, Serialize, Message)]
#[rtype(result = "()")]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RegistrationUpdateEnum {
  /// Broadcasted whenever a player registers/unregisters and before min players reached
  #[serde(rename_all = "camelCase")]
  WaitingOnPlayers {
    players: HashMap<Uuid, JWTPlayerData>,
    min_players_needed: usize,
    max_players_allowed: usize,
  },

  /// Game has minimum number of players and will start soon
  #[serde(rename_all = "camelCase")]
  GameStartingSoon {
    players: HashMap<Uuid, JWTPlayerData>,
    min_players_needed: usize,
    max_players_allowed: usize,
    seconds_left: u32,
  },

  /// Game is starting NOW!
  #[serde(rename_all = "camelCase")]
  GameStarting {
    players: HashMap<Uuid, JWTPlayerData>,
    player_order: Vec<Uuid>,
  },
}
