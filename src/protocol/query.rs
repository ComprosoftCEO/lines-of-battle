use serde::{Deserialize, Serialize};

/// Get the (x,y) position of a player in the grid
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerDetails {
  pub x: u32,
  pub y: u32,
  pub health: u32,
  pub current_weapon: Option<WeaponDetails>,
}

/// Weapon details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeaponDetails {
  pub name: String,
  pub damage_per_hit: u32,
  pub ammo_left: u32,
}
