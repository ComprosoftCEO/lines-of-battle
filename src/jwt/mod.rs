//
// Structures and functions related to JSON web tokens
//
pub mod audience;
mod jwt_data;
mod jwt_secret;
mod jwt_token;
mod websocket_token;

pub use audience::Audience;
pub use jwt_data::*;
pub use jwt_secret::JWTSecret;
pub use jwt_token::*;
pub use websocket_token::*;

// Other JWT constants
pub const JWT_ISSUER: &str = "game-server";
pub const JWT_EXPIRATION_MIN: i64 = 10;

// Type aliases for the different JWT tokens
pub type PlayerToken = JWTToken<audience::Player, JWTPlayerData>;
pub type ViewerToken = JWTToken<audience::Viewer, ()>;

/// Type aliases for the different JWT websocket tokens
pub type PlayerWebsocketToken = JWTWebsocketToken<audience::Player, JWTPlayerData>;
pub type ViewerWebsocketToken = JWTWebsocketToken<audience::Viewer, ()>;
