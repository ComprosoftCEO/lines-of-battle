use chrono::Duration;
use chrono_english::{parse_duration, DateResult, Interval};
use dotenv::dotenv;
use std::env;
use structopt::StructOpt;
use uuid::Uuid;

use game_server::jwt::{JWTPlayerData, JWTSecret, PlayerToken, ViewerToken};

/// Generate a JSON web token for the game server
#[derive(StructOpt)]
enum Opt {
  /// Generate a player JWT
  Player {
    /// Player UUID (Picks a random one if omitted))
    #[structopt(short, long)]
    id: Option<Uuid>,

    /// Player name or alias
    #[structopt(short, long)]
    name: String,

    /// Duration for the JWT as an English string
    #[structopt(short, long, default_value = "1 year")]
    duration: String,

    /// JSON Web Token secret
    #[structopt(short = "s", long, env, hide_env_values = true)]
    jwt_secret: String,
  },

  /// Generate a viewer JWT
  Viewer {
    /// Viewer UUID (Picks a random one if omitted))
    #[structopt(short, long)]
    id: Option<Uuid>,

    /// Duration for the JWT as an English string
    #[structopt(short, long, default_value = "1 year")]
    duration: String,

    /// JSON Web Token secret
    #[structopt(short = "s", long, env, hide_env_values = true)]
    jwt_secret: String,
  },
}

impl Opt {
  /// Returns the ID, or generate a new random ID if none is provided
  ///   The second parameter is "true" if a new ID was generated
  pub fn get_id(&self) -> (Uuid, bool) {
    let id = match self {
      Self::Player { id, .. } => id,
      Self::Viewer { id, .. } => id,
    };

    let new_id_generated = id.is_none();
    (id.unwrap_or_else(|| Uuid::new_v4()), new_id_generated)
  }

  /// Get the read JSON Web Token secret
  pub fn get_jwt_secret(&self) -> &String {
    match self {
      Self::Player { jwt_secret, .. } => jwt_secret,
      Self::Viewer { jwt_secret, .. } => jwt_secret,
    }
  }

  /// Update the environment variables with the command-line options
  pub fn update_environment(&self) {
    env::set_var("JWT_SECRET", self.get_jwt_secret());
  }

  /// Parse the duration or return an error
  pub fn parse_duration(&self) -> DateResult<Duration> {
    let duration = match self {
      Self::Player { duration, .. } => parse_duration(duration),
      Self::Viewer { duration, .. } => parse_duration(duration),
    }?;

    Ok(match duration {
      Interval::Seconds(secs) => Duration::days(secs as i64),
      Interval::Days(days) => Duration::days(days as i64),
      Interval::Months(months) => Duration::days((months as i64) * 30),
    })
  }
}

//
// Main program entry point
//
fn main() -> anyhow::Result<()> {
  // Load our ".env" configuration file
  dotenv().ok();

  // Parse command-line arguments
  let opt: Opt = Opt::from_args();
  opt.update_environment();

  // Parse the duration
  let duration = opt
    .parse_duration()
    .or_else(|e| Err(anyhow::anyhow!("invalid duration: {}", e)))?;

  let jwt_encoding_key = JWTSecret::new(opt.get_jwt_secret()).get_encoding_key();
  let (id, new_id_generated) = opt.get_id();

  // Generate and encode the token
  let token = match opt {
    Opt::Player { name, .. } => {
      let token = PlayerToken::new(id, duration, JWTPlayerData::new(name));
      token
        .encode(&jwt_encoding_key)
        .or_else(|e| Err(anyhow::anyhow!("failed to encode JWT: {}", e)))?
    },

    Opt::Viewer { .. } => {
      let token = ViewerToken::new(id, duration, ());
      token
        .encode(&jwt_encoding_key)
        .or_else(|e| Err(anyhow::anyhow!("failed to encode JWT: {}", e)))?
    },
  };

  // Print the token UUID to standard error if a random one was generated
  if new_id_generated {
    eprintln!("Token UUID: {}", id);
  }
  println!("{}", token);

  Ok(())
}
