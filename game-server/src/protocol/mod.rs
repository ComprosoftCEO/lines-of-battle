//
// Data structures to faciliate communication to the game
//
pub mod actions;
pub mod game;
pub mod query;
pub mod registration;
pub mod tagged_request;
pub mod websocket;

pub use actions::PlayerAction;
pub use game::{GameState, GameStateUpdate};
pub use query::QueryResponse;
pub use registration::RegistrationUpdateEnum;
pub use tagged_request::TaggedRequest;
pub use websocket::{ViewerMessage, WebsocketMessage};

use bytestring::ByteString;
use serde::Serialize;

/// Helpful trait to convert a serializable type into a ByteString
pub trait ToBytestring {
  /// Serialize the object into a bytestring
  fn to_bytestring(&self) -> ByteString;

  /// Consume the object and convert into a bytestring
  fn into_bytestring(self) -> ByteString
  where
    Self: Sized,
  {
    self.to_bytestring()
  }
}

impl<T> ToBytestring for T
where
  T: Serialize,
{
  fn to_bytestring(&self) -> ByteString {
    serde_json::to_string(&self).unwrap().into()
  }
}
