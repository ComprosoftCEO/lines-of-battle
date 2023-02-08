use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

use crate::game::ServerState;
use crate::jwt::JWTPlayerData;

/// List of all responses to a query
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum QueryResponse {
  #[serde(rename_all = "camelCase")]
  ServerState { state: ServerState },

  #[serde(rename_all = "camelCase")]
  RegisteredPlayers {
    players: HashMap<Uuid, JWTPlayerData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    player_order: Option<Vec<Uuid>>,
  },
}
