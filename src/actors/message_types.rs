use actix::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

use crate::actors::WebsocketActor;
use crate::protocol::{GameStateUpdate, PlayerAction};

/// Test if the game is currently running
#[derive(Debug, Clone, Message)]
#[rtype(result = "bool")]
pub struct IsGameRunning;

/// Test if a user is registered
#[derive(Debug, Clone, Message)]
#[rtype(result = "bool")]
pub struct IsPlayerRegistered(pub Uuid);

/// Test if a user is connected
#[derive(Debug, Clone, Message)]
#[rtype(result = "bool")]
pub struct IsPlayerConnected(pub Uuid);

/// Connect a user with the mediator
#[derive(Debug, Clone, Message)]
#[rtype(result = "ConnectResponse")]
pub struct Connect(pub Uuid, pub Addr<WebsocketActor>);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Message, MessageResponse)]
#[rtype(result = "()")]
pub enum ConnectResponse {
  Ok,
  NotRegistered,
  AlreadyConnected,
}

/// Disconnect a user from the mediator
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct Disconnect(pub Uuid);

/// Fatal error has caused the game engine to crash - Server must reboot!
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct GameEngineCrash;
