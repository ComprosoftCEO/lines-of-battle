use actix::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::actors::{ViewerActor, WebsocketActor};
use crate::game::ServerState;
use crate::jwt::JWTPlayerData;

/// Connect a websocket actor with the mediator
#[derive(Debug, Clone, Message)]
#[rtype(result = "ConnectResponse")]
pub struct Connect(pub Uuid, pub Addr<WebsocketActor>);

/// Response from the connection
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Message, MessageResponse)]
#[rtype(result = "()")]
pub enum ConnectResponse {
  Ok(ServerState),
  NotRegistered,
  AlreadyConnected,
}

/// Disconnect a websocket actor from the mediator
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct Disconnect(pub Uuid, pub Addr<WebsocketActor>);

/// Connect a viewer actor with the mediator
#[derive(Debug, Clone, Message)]
#[rtype(result = "ConnectViewerResponse")]
pub struct ConnectViewer(pub Addr<ViewerActor>);

/// Response from the viewer connection
#[derive(Debug, Clone, Message, MessageResponse)]
#[rtype(result = "()")]
pub struct ConnectViewerResponse(pub ServerState);

/// Disconnect a viewer actor from the mediator
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct DisconnectViewer(pub Addr<ViewerActor>);

/// Register a player in the game -- This is idempotent
#[derive(Debug, Clone, Message)]
#[rtype(result = "RegisterResponse")]
pub struct Register {
  pub id: Uuid,
  pub data: JWTPlayerData,
}

#[derive(Debug, Clone, Copy, MessageResponse)]
pub enum RegisterResponse {
  Success,
  GameAlreadyStarted,
  TooManyRegistered { max_allowed: usize },
}

/// Unregister a player from the game -- This is idempotent
///  Returns true to indicate player is marked as not registered
///  Returns false if game is started and player is registered
#[derive(Debug, Clone, Message)]
#[rtype(result = "bool")]
pub struct Unregister {
  pub id: Uuid,
}

/// Get the list of registered players from the mediator
#[derive(Debug, Clone, Message)]
#[rtype(result = "GetRegisteredPlayersResponse")]
pub struct GetRegisteredPlayers;

#[derive(Debug, Clone, MessageResponse)]
pub struct GetRegisteredPlayersResponse {
  pub players: HashMap<Uuid, JWTPlayerData>,
  pub player_order: Option<Vec<Uuid>>,
}
