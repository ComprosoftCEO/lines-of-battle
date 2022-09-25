use serde::{Deserialize, Serialize};

use crate::protocol::actions::*;

/// Every request can include an optional tag, used by the clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaggedRequest<T> {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tag: Option<String>,

  #[serde(flatten)]
  pub data: T,
}

impl TaggedRequest<MoveAction> {
  pub fn transpose(self) -> TaggedRequest<PlayerActionEnum> {
    TaggedRequest {
      tag: self.tag,
      data: PlayerActionEnum::Move(self.data),
    }
  }
}

impl TaggedRequest<AttackAction> {
  pub fn transpose(self) -> TaggedRequest<PlayerActionEnum> {
    TaggedRequest {
      tag: self.tag,
      data: PlayerActionEnum::Attack(self.data),
    }
  }
}

impl TaggedRequest<Option<()>> {
  pub fn transpose(self) -> TaggedRequest<PlayerActionEnum> {
    TaggedRequest {
      tag: self.tag,
      data: PlayerActionEnum::DropWeapon,
    }
  }
}
