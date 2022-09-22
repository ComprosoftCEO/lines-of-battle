use serde::Deserialize;
use std::collections::HashSet;
use std::iter::FromIterator;

/// Route is only available to player clients
#[derive(Deserialize)]
pub struct Player;

/// Route is only available to viewer clients
#[derive(Deserialize)]
pub struct Viewer;

/// Generic trait shared by all audience types
///
/// An audience specifies which routes a JWT can access
pub trait Audience {
  const TEXT: &'static str;
  const ACCEPTS: &'static [&'static str];

  /// Get the name to show for this audience type
  fn get_name() -> String {
    Self::TEXT.to_string()
  }

  /// Get the list of all audiences accepted by this type
  fn accepts() -> HashSet<String> {
    HashSet::from_iter(Self::ACCEPTS.iter().cloned().map(String::from))
  }
}

impl Audience for Player {
  const TEXT: &'static str = "player";
  const ACCEPTS: &'static [&'static str] = &["player"];
}

impl Audience for Viewer {
  const TEXT: &'static str = "viewer";
  const ACCEPTS: &'static [&'static str] = &["viewer"];
}
