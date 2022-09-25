use actix::prelude::*;
use bytestring::ByteString;
use uuid::Uuid;

/// Sent to the websocket actor if the game starts and they are not registered
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct KickUnregisteredPlayer;

/// Broadcast update about registration before the game has started
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct RegistrationUpdate(pub ByteString);

/// Game is now being initialized, registration is permenantly closed
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct GameStarting(pub ByteString);

/// Broadcast the init message with the first game state
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct Init(pub ByteString);

/// Broadcast the next state message
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct NextState(pub ByteString);

/// Broadcast the player killed message
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct PlayerKilled(pub Uuid, pub ByteString);

/// Broadcast the game ended message
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct GameEnded(pub ByteString);
