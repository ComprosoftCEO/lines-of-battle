//
// Messages to send to the websocket actor
//
use actix::prelude::*;

/// Sent to the websocket actor to close the connection
///   This happens if the game starts and they are not registered
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct KickUnregisteredPlayer;
