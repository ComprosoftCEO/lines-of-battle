use serde_repr::{Deserialize_repr, Serialize_repr};

/// Error codes that are exposed to the client
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum GlobalErrorCode {
  UnknownError = 0,
  MissingAppData,
  JSONPayloadError,
  FormPayloadError,
  URLPathError,
  QueryStringError,
  StructValidationError,
  InvalidJWTToken,
  GameEngineError,
  GameEngineCrash,
  WebsocketError,
  NotRegistered,
  FailedToRegister,
  FailedToUnregister,
  AlreadyConnected,
  CannotSendAction,
}
