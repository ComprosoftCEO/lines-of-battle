//
// Environment configuration functions
//
use dotenv::dotenv;
use std::any::type_name;
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 53700;
const DEFAULT_JWT_SECRET: &str = "secret";

pub const DEFAULT_LUA_FILE: &str = "lua/game.lua";

const DEFAULT_MIN_PLAYERS: usize = 2;
const DEFAULT_MAX_PLAYERS: usize = 8;
const DEFAULT_LOBBY_WAIT_SECONDS: u32 = 10;
const DEFAULT_TICK_PER_GAME: u32 = 60 * 3;
const DEFAULT_SECONDS_PER_TICK: u32 = 1;

/// API Game Server for the Semester Project
#[derive(StructOpt)]
pub struct Opt {
  /// Host to run the server
  #[structopt(short, long, env, default_value = DEFAULT_HOST)]
  host: String,

  /// Port to use for the server
  #[structopt(short, long, env, default_value = "53700")]
  port: u16,

  /// Enable HTTPS (SSL) for the server
  #[structopt(long, env, takes_value(false), requires("key-file"), requires("cert-file"))]
  use_https: bool,

  /// Path for the SSL private key file
  #[structopt(long, env, parse(from_os_str))]
  key_file: Option<PathBuf>,

  /// Path for the SSL certificate chain file
  #[structopt(long, env, parse(from_os_str))]
  cert_file: Option<PathBuf>,

  /// JSON Web Token secret
  #[structopt(short = "s", long, env, hide_env_values = true, default_value = DEFAULT_JWT_SECRET, hide_default_value(true))]
  jwt_secret: String,

  /// Lua file containing the game engine code
  #[structopt(long, env, default_value = DEFAULT_LUA_FILE)]
  lua_file: String,

  /// Minimum number of players required to play the game
  #[structopt(long, env, default_value = "2")]
  min_players_needed: usize,

  /// Maximum number of players allowed to play in the game
  #[structopt(long, env, default_value = "8")]
  max_players_allowed: usize,

  /// Amount of time to wait before starting the game after the minimum number of players is reached
  #[structopt(long, env, default_value = "10")]
  lobby_wait_seconds: u32,

  /// Number of total "ticks" for a complete round in the game
  #[structopt(long, env, default_value = "180")]
  ticks_per_game: u32,

  /// Number of seconds between each "tick" in the game engine
  #[structopt(long, env, default_value = "1")]
  seconds_per_tick: u32,
}

impl Opt {
  /// Update the environment variables with the command-line options
  pub fn update_environment(&self) {
    env::set_var("HOST", &self.host);
    env::set_var("PORT", &self.port.to_string());

    if self.use_https {
      env::set_var("USE_HTTPS", "true");
    }
    if let Some(ref key_file) = self.key_file {
      env::set_var("KEY_FILE", key_file);
    }
    if let Some(ref cert_file) = self.cert_file {
      env::set_var("CERT_FILE", cert_file);
    }

    env::set_var("JWT_SECRET", &self.jwt_secret);
    env::set_var("LUA_FILE", &self.lua_file);

    env::set_var("MIN_PLAYERS_NEEDED", self.min_players_needed.to_string());
    env::set_var("MAX_PLAYERS_ALLOWED", self.max_players_allowed.to_string());
    env::set_var("LOBBY_WAIT_SECONDS", self.lobby_wait_seconds.to_string());
    env::set_var("TICKS_PER_GAME", self.ticks_per_game.to_string());
    env::set_var("SECONDS_PER_TICK", self.seconds_per_tick.to_string());
  }
}

/// Parse the string into the given type, returning a warning if the parsing failed
#[inline]
fn parse_with_warning<T>(env_name: &str, default_value: T) -> T
where
  T: FromStr + std::fmt::Display,
{
  let input = if let Ok(input) = env::var(env_name) {
    input
  } else {
    return default_value;
  };

  match input.parse() {
    Ok(value) => value,
    Err(_) => {
      log::warn!(
        "{}: invalid {} '{}', using default value '{}'",
        env_name,
        type_name::<T>(),
        input,
        default_value,
      );
      default_value
    },
  }
}

///
/// Load the .env files into the current environment
///
pub fn load_environment_from_env_files() {
  dotenv().ok(); /* .env file */
  if cfg!(debug_assertions) {
    dotenv::from_filename(".env.development").ok();
  } else {
    dotenv::from_filename(".env.production").ok();
  }
}

//
// Basic Server Variables
//
pub fn get_host() -> String {
  env::var("HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string())
}

pub fn get_port() -> u16 {
  parse_with_warning("PORT", DEFAULT_PORT)
}

//
// HTTPS and SSL/TLS Encryption
//
pub fn use_https() -> bool {
  parse_with_warning("USE_HTTPS", false)
}

pub fn get_key_file() -> Option<String> {
  env::var("KEY_FILE").ok()
}

pub fn get_cert_file() -> Option<String> {
  env::var("CERT_FILE").ok()
}

//
// JSON Web Token values
//
pub fn get_jwt_secret() -> String {
  env::var("JWT_SECRET").unwrap_or_else(|_| DEFAULT_JWT_SECRET.into())
}

//
// Lua engine code
//
pub fn get_lua_file() -> String {
  env::var("LUA_FILE").unwrap_or_else(|_| DEFAULT_LUA_FILE.into())
}

//
// Game Configuration Variables
//
pub fn get_min_players_needed() -> usize {
  let min_players = parse_with_warning("MIN_PLAYERS_NEEDED", DEFAULT_MIN_PLAYERS);
  if min_players < 2 {
    log::warn!("MIN_PLAYERS_NEEDED cannot be less than 2, using minimum value '2'");
    2
  } else {
    min_players
  }
}

pub fn get_max_players_allowed() -> usize {
  parse_with_warning("MAX_PLAYERS_ALLOWED", DEFAULT_MAX_PLAYERS)
}

pub fn get_lobby_wait_time_seconds() -> u32 {
  let lobby_wait_seconds = parse_with_warning("LOBBY_WAIT_SECONDS", DEFAULT_LOBBY_WAIT_SECONDS);
  if lobby_wait_seconds < 1 {
    log::warn!("LOBBY_WAIT_SECONDS cannot be less than 1, using minimum value '1'");
    1
  } else {
    lobby_wait_seconds
  }
}

pub fn get_ticks_per_game() -> u32 {
  let ticks_per_game = parse_with_warning("TICKS_PER_GAME", DEFAULT_TICK_PER_GAME);
  if ticks_per_game < 30 {
    log::warn!("TICKS_PER_GAME cannot be less than 30, using minimum value '30'");
    30
  } else {
    ticks_per_game
  }
}

pub fn get_seconds_per_tick() -> u32 {
  let seconds_per_tick = parse_with_warning("SECONDS_PER_TICK", DEFAULT_SECONDS_PER_TICK);
  if seconds_per_tick < 1 {
    log::warn!("SECONDS_PER_TICK cannot be less than 1, using minimum value '1'");
    1
  } else {
    seconds_per_tick
  }
}
