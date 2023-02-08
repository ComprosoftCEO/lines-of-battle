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
#[serde(rename_all = "camelCase")]
pub enum ServerState {
  Registration,
  Initializing,
  Running,
  FatalError,
}

impl ServerState {
  pub fn can_change_registration(&self) -> bool {
    use ServerState::*;

    match self {
      Registration => true,
      Initializing | Running | FatalError => false,
    }
  }

  pub fn can_send_action(&self) -> bool {
    use ServerState::*;

    match self {
      Running => true,
      Registration | Initializing | FatalError => false,
    }
  }
}
