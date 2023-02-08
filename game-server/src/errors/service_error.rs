use actix::MailboxError;
use actix_web::error::{JsonPayloadError, PathError, QueryPayloadError, UrlencodedError};
use actix_web::http::header::ToStrError;
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web_httpauth::headers::www_authenticate::bearer::Bearer;
use jsonwebtoken::errors::Error as JWTError;
use std::{error, fmt};
use uuid::Uuid;

use crate::errors::*;

/// Enumeration of all possible errors that can occur
#[derive(Debug)]
pub enum ServiceError {
  MissingAppData(String),
  JSONPayloadError(JsonPayloadError),
  FormPayloadError(UrlencodedError),
  URLPathError(PathError),
  QueryStringError(QueryPayloadError),
  JWTError(JWTError),
  JWTExtractorError(AuthenticationError<Bearer>),
  MissingWebsocketJWT,
  WebsocketJWTParseError(ToStrError),
  WebsocketError(WebsocketError),
  WebsocketMailboxError(MailboxError),
  NotRegistered(Uuid),
  FailedToRegister(Uuid, String),
  FailedToUnregister(Uuid),
  AlreadyConnected(Uuid),
  GameEngineError(GameEngineError),
  GameEngineCrash,
  CannotSendAction { why: String },
}

impl ServiceError {
  pub fn get_error_response(&self) -> ErrorResponse {
    match self {
      ServiceError::MissingAppData(data) => ErrorResponse::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Server Misconfiguration".into(),
        GlobalErrorCode::MissingAppData,
        format!("'{}' not configured using App::data()", data),
      ),

      ServiceError::JSONPayloadError(error) => ErrorResponse::new(
        StatusCode::BAD_REQUEST,
        "Invalid JSON Object".into(),
        GlobalErrorCode::JSONPayloadError,
        format!("{}", error),
      ),

      ServiceError::FormPayloadError(error) => ErrorResponse::new(
        StatusCode::BAD_REQUEST,
        "Invalid Form Data".into(),
        GlobalErrorCode::FormPayloadError,
        format!("{}", error),
      ),

      ServiceError::URLPathError(error) => ErrorResponse::new(
        StatusCode::BAD_REQUEST,
        "Invalid URL Path".into(),
        GlobalErrorCode::URLPathError,
        format!("{}", error),
      ),

      ServiceError::QueryStringError(error) => ErrorResponse::new(
        StatusCode::BAD_REQUEST,
        "Invalid Query String".into(),
        GlobalErrorCode::QueryStringError,
        format!("{}", error),
      ),

      ServiceError::JWTError(error) => ErrorResponse::new(
        StatusCode::UNAUTHORIZED,
        "Invalid JWT Token".into(),
        GlobalErrorCode::InvalidJWTToken,
        format!("{}", error),
      ),

      ServiceError::JWTExtractorError(error) => ErrorResponse::new(
        StatusCode::UNAUTHORIZED,
        "Invalid JWT Token".into(),
        GlobalErrorCode::InvalidJWTToken,
        format!("{}", error),
      ),

      ServiceError::MissingWebsocketJWT => ErrorResponse::new(
        StatusCode::UNAUTHORIZED,
        "Invalid JWT Token".into(),
        GlobalErrorCode::InvalidJWTToken,
        "Missing JWT token in 'Sec-WebSocket-Protocol' header".into(),
      ),

      ServiceError::WebsocketJWTParseError(error) => ErrorResponse::new(
        StatusCode::UNAUTHORIZED,
        "Invalid JWT Token".into(),
        GlobalErrorCode::InvalidJWTToken,
        format!("{}", error),
      ),

      ServiceError::WebsocketError(error) => ErrorResponse::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Unexpected websocket error".into(),
        GlobalErrorCode::WebsocketError,
        format!("{:#?}", error),
      ),

      ServiceError::WebsocketMailboxError(error) => ErrorResponse::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Actix message error".into(),
        GlobalErrorCode::WebsocketError,
        format!("{:?}", error),
      ),

      ServiceError::NotRegistered(player_id) => ErrorResponse::new(
        StatusCode::CONFLICT,
        "Player not registered to play in the game".into(),
        GlobalErrorCode::NotRegistered,
        format!("Player ID: {}", player_id),
      ),

      ServiceError::FailedToRegister(player_id, why) => ErrorResponse::new(
        StatusCode::CONFLICT,
        format!("Failed to register player: {}", why),
        GlobalErrorCode::FailedToRegister,
        format!("Player ID: {}", player_id),
      ),

      ServiceError::FailedToUnregister(player_id) => ErrorResponse::new(
        StatusCode::CONFLICT,
        "Failed to unregister player".into(),
        GlobalErrorCode::FailedToUnregister,
        format!("Player ID: {}", player_id),
      ),

      ServiceError::AlreadyConnected(player_id) => ErrorResponse::new(
        StatusCode::CONFLICT,
        "Player already connected on another websocket".into(),
        GlobalErrorCode::AlreadyConnected,
        format!("Player ID: {}", player_id),
      ),

      ServiceError::GameEngineError(error) => ErrorResponse::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal game engine error".into(),
        GlobalErrorCode::GameEngineError,
        error.get_developer_notes(),
      ),

      ServiceError::GameEngineCrash => ErrorResponse::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Game engine has crashed".into(),
        GlobalErrorCode::GameEngineCrash,
        "".into(),
      ),

      ServiceError::CannotSendAction { why } => ErrorResponse::new(
        StatusCode::CONFLICT,
        format!("Cannot send action: {}", why),
        GlobalErrorCode::CannotSendAction,
        "".into(),
      ),
    }
  }
}

//
// Various Error Traits
//
impl fmt::Display for ServiceError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self.get_error_response())
  }
}

impl ResponseError for ServiceError {
  fn error_response(&self) -> HttpResponse {
    let error = self.get_error_response();
    log::error!("{:?}", error);
    error.error_response()
  }
}

impl error::Error for ServiceError {}

//
// Implicit conversion functions
//
impl From<JsonPayloadError> for ServiceError {
  fn from(error: JsonPayloadError) -> Self {
    ServiceError::JSONPayloadError(error)
  }
}

impl From<UrlencodedError> for ServiceError {
  fn from(error: UrlencodedError) -> Self {
    ServiceError::FormPayloadError(error)
  }
}

impl From<PathError> for ServiceError {
  fn from(error: PathError) -> Self {
    ServiceError::URLPathError(error)
  }
}

impl From<QueryPayloadError> for ServiceError {
  fn from(error: QueryPayloadError) -> Self {
    ServiceError::QueryStringError(error)
  }
}

impl From<JWTError> for ServiceError {
  fn from(error: JWTError) -> Self {
    ServiceError::JWTError(error)
  }
}

impl From<AuthenticationError<Bearer>> for ServiceError {
  fn from(error: AuthenticationError<Bearer>) -> Self {
    ServiceError::JWTExtractorError(error)
  }
}

impl From<WebsocketError> for ServiceError {
  fn from(error: WebsocketError) -> Self {
    ServiceError::WebsocketError(error)
  }
}

impl From<GameEngineError> for ServiceError {
  fn from(error: GameEngineError) -> Self {
    ServiceError::GameEngineError(error)
  }
}
