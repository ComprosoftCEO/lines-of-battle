use serde::Serialize;

use crate::game::GameState;

/// List of all responses to a query
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum QueryResponse {
  #[serde(rename_all = "camelCase")]
  CurrentState { state: GameState },
}
