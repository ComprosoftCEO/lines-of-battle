use actix::prelude::*;

/// Fatal error has caused the game engine to crash - Server must reboot!
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct GameEngineCrash;
