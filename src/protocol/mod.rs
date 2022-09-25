//
// Data structures to faciliate communication to the game
//
pub mod actions;
pub mod game;
pub mod registration;
pub mod tagged_request;
pub mod websocket;

pub use actions::PlayerAction;
pub use game::GameStateUpdate;
pub use registration::RegistrationUpdateEnum;
pub use tagged_request::TaggedRequest;
pub use websocket::WebsocketMessage;

use bytestring::ByteString;
use serde::Serialize;

pub trait ToBytestring {
  fn to_bytestring(&self) -> serde_json::Result<ByteString>;
}

impl<T> ToBytestring for T
where
  T: Serialize,
{
  fn to_bytestring(&self) -> serde_json::Result<ByteString> {
    Ok(serde_json::to_string(&self)?.into())
  }
}
