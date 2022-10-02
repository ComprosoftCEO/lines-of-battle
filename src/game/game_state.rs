use serde::{Deserialize, Serialize};

/// Represents the state transitions in the game engine
///
/// ```text
///  Registration --> Initializing --> Running
///    ^                                  V
///    \--<-----------<--------------<----/
/// ```
///
/// All states can go to a fatal error
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameState {
  Registration,
  Initializing,
  Running,
  FatalError,
}

impl GameState {
  pub fn can_change_registration(&self) -> bool {
    use GameState::*;

    match self {
      Registration => true,
      Initializing | Running | FatalError => false,
    }
  }

  pub fn can_send_action(&self) -> bool {
    use GameState::*;

    match self {
      Running => true,
      Registration | Initializing | FatalError => false,
    }
  }
}
