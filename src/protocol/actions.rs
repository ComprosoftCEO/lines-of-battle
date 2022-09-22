use serde::{Deserialize, Serialize};

/// List of all actions the player can take in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AllPlayerActions {
  Move(MoveAction),
  Attack(AttackAction),
  DropWeapon,
}

/// Cardinal direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Direction {
  Up,
  Down,
  Left,
  Right,
}

/// Move the player in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveAction {
  pub direction: Direction,
}

/// Attack / Shoot in a given direction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttackAction {
  pub direction: Direction,
}
