use actix::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use uuid::Uuid;

use crate::protocol::PlayerAction;

/// Notify the mediator that the game state has been updated
#[derive(Debug, Clone, Serialize, Message)]
#[rtype(result = "()")]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GameStateUpdate {
  /// Game has just been initialized (returns the initial game state)
  #[serde(rename_all = "camelCase")]
  Init {
    game_state: serde_json::Value,
    seconds_left: u32,
  },

  /// Game has been updated
  #[serde(rename_all = "camelCase")]
  NextState {
    game_state: serde_json::Value,
    actions_taken: HashMap<Uuid, PlayerAction>,
    seconds_left: u32,
  },

  /// Sent every time a player is killed
  #[serde(rename_all = "camelCase")]
  PlayerKilled { id: Uuid },

  /// Sent when the game is over (returns the final game state)
  #[serde(rename_all = "camelCase")]
  GameEnded {
    winners: HashSet<Uuid>,
    game_state: serde_json::Value,
    actions_taken: HashMap<Uuid, PlayerAction>,
  },
}
