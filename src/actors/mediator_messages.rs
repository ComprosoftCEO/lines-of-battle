use actix::prelude::*;
use uuid::Uuid;

use crate::actors::WebsocketActor;
use crate::game::GameState;
use crate::jwt::JWTPlayerData;

/// Connect a websocket actor with the mediator
#[derive(Debug, Clone, Message)]
#[rtype(result = "ConnectResponse")]
pub struct Connect(pub Uuid, pub Addr<WebsocketActor>);

/// Response from the connection
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Message, MessageResponse)]
#[rtype(result = "()")]
pub enum ConnectResponse {
  Ok(GameState),
  NotRegistered,
  AlreadyConnected,
}

/// Disconnect a websocket actor from the mediator
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct Disconnect(pub Uuid, pub Addr<WebsocketActor>);

/// Register a player in the game -- This is idempotent
///  Returns true to indicate player is marked as registered
///  Returns false if game is started and player is not registered
#[derive(Debug, Clone, Message)]
#[rtype(result = "bool")]
pub struct Register {
  pub id: Uuid,
  pub data: JWTPlayerData,
}

/// Unregister a player from the game -- This is idempotent
///  Returns true to indicate player is marked as not registered
///  Returns false if game is started and player is registered
#[derive(Debug, Clone, Message)]
#[rtype(result = "bool")]
pub struct Unregister {
  pub id: Uuid,
}
