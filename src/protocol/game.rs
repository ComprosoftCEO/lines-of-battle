use actix::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::protocol::PlayerAction;

/// Notify the mediator that the game state has been updated
#[derive(Debug, Clone, Serialize, Message)]
#[rtype(result = "()")]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GameStateUpdate {
  /// Game has just been initialized (returns the initial game state)
  #[serde(rename_all = "camelCase")]
  Init { game_state: GameState, seconds_left: u32 },

  /// Game has been updated
  #[serde(rename_all = "camelCase")]
  NextState {
    game_state: GameState,
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
    game_state: GameState,
    actions_taken: HashMap<Uuid, PlayerAction>,
  },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
  playfield: Vec<Vec<u32>>,
  players: HashMap<Uuid, serde_json::Map<String, serde_json::Value>>,
  weapons: Vec<serde_json::Map<String, serde_json::Value>>,
}
