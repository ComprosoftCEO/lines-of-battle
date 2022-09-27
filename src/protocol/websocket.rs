use serde::Deserialize;

use crate::protocol::actions::{AttackAction, DropWeaponAction, MoveAction};
use crate::protocol::TaggedRequest;

/// List of all messages that the player can sent to the WebSocket
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WebsocketMessage {
  // Registration
  Register,
  Unregister,

  // Player actions
  Move(TaggedRequest<MoveAction>),
  Attack(TaggedRequest<AttackAction>),
  DropWeapon(TaggedRequest<DropWeaponAction>),
}
