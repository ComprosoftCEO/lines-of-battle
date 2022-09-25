//
// Data structures to faciliate communication to the game
//
pub mod actions;
pub mod game;

pub use actions::PlayerAction;
pub use game::GameStateUpdate;

use serde::{Deserialize, Serialize};

/// Every request can include an optional tag, used by the clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolRequest<T> {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tag: Option<String>,

  #[serde(flatten)]
  pub data: T,
}
