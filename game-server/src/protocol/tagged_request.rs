use serde::{Deserialize, Serialize};

use crate::protocol::actions::*;

/// Some requests can include an optional tag, used by the clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaggedRequest<T> {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tag: Option<String>,

  #[serde(flatten)]
  pub data: T,
}

impl<T> TaggedRequest<T> {
  pub fn new(data: T) -> Self {
    Self { data, tag: None }
  }

  pub fn new_tagged(data: T, tag: impl Into<String>) -> Self {
    Self {
      data,
      tag: Some(tag.into()),
    }
  }
}

impl TaggedRequest<MoveAction> {
  /// Move the tag to an outside data structure
  pub fn transpose(self) -> TaggedRequest<PlayerActionEnum> {
    TaggedRequest {
      tag: self.tag,
      data: PlayerActionEnum::Move(self.data),
    }
  }
}

impl TaggedRequest<AttackAction> {
  /// Move the tag to an outside data structure
  pub fn transpose(self) -> TaggedRequest<PlayerActionEnum> {
    TaggedRequest {
      tag: self.tag,
      data: PlayerActionEnum::Attack(self.data),
    }
  }
}

impl TaggedRequest<DropWeaponAction> {
  /// Move the tag to an outside data structure
  pub fn transpose(self) -> TaggedRequest<PlayerActionEnum> {
    TaggedRequest {
      tag: self.tag,
      data: PlayerActionEnum::DropWeapon,
    }
  }
}
