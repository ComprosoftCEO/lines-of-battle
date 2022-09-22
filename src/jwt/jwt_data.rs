use serde::{Deserialize, Serialize};

/// Other fields used by JWT for players
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JWTPlayerData {
  name: String,
}

impl JWTPlayerData {
  pub fn new(name: impl Into<String>) -> Self {
    Self { name: name.into() }
  }

  pub fn get_name(&self) -> &String {
    &self.name
  }
}
