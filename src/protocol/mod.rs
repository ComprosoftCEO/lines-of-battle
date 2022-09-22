//
// Data structures to faciliate communication to the game
//
pub mod actions;
pub mod query;

use serde::Deserialize;

/// Every request can include an optional tag, used by the clients
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request<T> {
  pub tag: Option<String>,

  #[serde(flatten)]
  pub data: T,
}
