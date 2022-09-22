use actix::prelude::*;
use uuid::Uuid;

/// Sent to the websocket actors on a game state update
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub enum GameStateUpdate {
  GameStart,
  StateUpdate,
  GameOver,
}

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
pub struct Connect(pub Uuid, pub Recipient<GameStateUpdate>);

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
