use serde::{Deserialize, Serialize};

/// Other fields used by JWT for players
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JWTPlayerData {
  name: String,
}
