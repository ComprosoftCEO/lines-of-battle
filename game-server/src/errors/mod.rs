//
// All code related to error handling for the API server
//
mod error_response;
mod game_engine_error;
mod global_error_codes;
mod service_error;
mod websocket_error;

pub use error_response::ErrorResponse;
pub use game_engine_error::GameEngineError;
pub use global_error_codes::GlobalErrorCode;
pub use service_error::ServiceError;
pub use websocket_error::WebsocketError;
