use crate::protocol::TaggedRequest;
use serde::{Deserialize, Serialize};

/// PlayerAction with an optional associated tag
pub type PlayerAction = TaggedRequest<PlayerActionEnum>;

/// Enum of the actual actions taken
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PlayerActionEnum {
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

/// Drop a weapon - Just declare an empty struct
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DropWeaponAction {}
