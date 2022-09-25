use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::errors::GlobalErrorCode;

/// JSON response returned on an error
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
  rename_all = "camelCase",
  into = "WrappedErrorResponse",
  from = "WrappedErrorResponse"
)]
pub struct ErrorResponse {
  status_code: StatusCode,
  description: String,
  error_code: GlobalErrorCode,
  developer_notes: Option<String>,
}

/// Helper type so the JSON has the `"type": "error"` JSON tag
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum WrappedErrorResponse {
  #[serde(rename_all = "camelCase")]
  Error {
    description: String,
    error_code: GlobalErrorCode,

    // Don't serialize on production system
    #[cfg_attr(not(debug_assertions), serde(skip_serializing))]
    #[serde(skip_serializing_if = "Option::is_none")]
    developer_notes: Option<String>,
  },
}

impl From<WrappedErrorResponse> for ErrorResponse {
  fn from(error: WrappedErrorResponse) -> Self {
    match error {
      WrappedErrorResponse::Error {
        description,
        error_code,
        developer_notes,
      } => Self {
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
        description,
        error_code,
        developer_notes,
      },
    }
  }
}

impl From<ErrorResponse> for WrappedErrorResponse {
  fn from(error: ErrorResponse) -> Self {
    Self::Error {
      description: error.description,
      error_code: error.error_code,
      developer_notes: error.developer_notes,
    }
  }
}

impl ErrorResponse {
  pub fn new(
    status_code: StatusCode,
    description: String,
    error_code: GlobalErrorCode,
    developer_notes: String,
  ) -> Self {
    ErrorResponse {
      status_code,
      description,
      error_code,
      developer_notes: Some(developer_notes),
    }
  }

  pub fn get_status_code(&self) -> StatusCode {
    self.status_code
  }

  pub fn get_description(&self) -> &String {
    &self.description
  }

  pub fn get_error_code(&self) -> &GlobalErrorCode {
    &self.error_code
  }

  pub fn get_developer_notes(&self) -> Option<&String> {
    self.developer_notes.as_ref()
  }
}

impl fmt::Display for ErrorResponse {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "(#{:?}): {}", self.error_code, self.description)?;
    if let Some(ref notes) = self.developer_notes {
      write!(f, "\nDeveloper Notes: {}", notes)?;
    }
    Ok(())
  }
}

impl ResponseError for ErrorResponse {
  fn error_response(&self) -> HttpResponse {
    HttpResponse::build(self.status_code).json(&self)
  }
}
