use actix::Actor;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use log::LevelFilter;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use simple_logger::SimpleLogger;
use std::{fs::File, io::BufReader};
use structopt::StructOpt;

use game_server::config;
use game_server::errors::ServiceError;
use game_server::jwt::JWTSecret;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
  // Parse ".env" configuration files and command-line arguments
  config::load_environment_from_env_files();

  let opt = config::Opt::from_args();
  opt.update_environment();

  // Configure the logger system
  SimpleLogger::new().init()?;
  if cfg!(debug_assertions) {
    log::set_max_level(LevelFilter::Debug);
  } else {
    log::set_max_level(LevelFilter::Info);
  }

  // Database connection pool and web server
  let mut server = HttpServer::new(move || {
    App::new()
      // Secret key for JSON Web Tokens
      .app_data(web::Data::new(JWTSecret::new(config::get_jwt_secret())))
      // Enable logger
      .wrap(middleware::Logger::default())
      // Configure error handlers
      .app_data(web::JsonConfig::default().error_handler(|err, _req| ServiceError::from(err).into()))
      .app_data(web::FormConfig::default().error_handler(|err, _req| ServiceError::from(err).into()))
      .app_data(web::PathConfig::default().error_handler(|err, _req| ServiceError::from(err).into()))
      .app_data(web::QueryConfig::default().error_handler(|err, _req| ServiceError::from(err).into()))
      // Load all routes
      .default_service(web::route().to(|| HttpResponse::NotFound()))
  });

  // Possibly enable SSL
  let ip_port = format!("{}:{}", config::get_host(), config::get_port());
  server = if config::use_https() {
    server.bind_rustls(ip_port, get_ssl_configuration()?)?
  } else {
    server.bind(ip_port)?
  };

  // Run and listen for connections
  Ok(server.run().await?)
}

///
/// Load and configure SSL if required
///
fn get_ssl_configuration() -> anyhow::Result<ServerConfig> {
  let key_filename = config::get_key_file().ok_or_else(|| anyhow::anyhow!("KEY_FILE environment variable not set"))?;
  let cert_filename =
    config::get_cert_file().ok_or_else(|| anyhow::anyhow!("CERT_FILE environment variable not set"))?;

  // Init server config builder with safe defaults
  let config = ServerConfig::builder().with_safe_defaults().with_no_client_auth();

  // Read the TLS key/cert files
  let cert_file = &mut BufReader::new(
    File::open(&key_filename).or_else(|e| Err(anyhow::anyhow!("Failed to open '{}': {}", key_filename, e)))?,
  );
  let key_file = &mut BufReader::new(
    File::open(&cert_filename).or_else(|e| Err(anyhow::anyhow!("Failed to open '{}': {}", cert_filename, e)))?,
  );

  // Convert files to key/cert objects
  let cert_chain = certs(cert_file)?.into_iter().map(Certificate).collect();
  let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)?.into_iter().map(PrivateKey).collect();

  // Exit if no keys could be parsed
  if keys.is_empty() {
    Err(anyhow::anyhow!("Could not locate PKCS 8 private keys."))?;
  }

  let config = config.with_single_cert(cert_chain, keys.remove(0))?;
  log::debug!("Loaded SSL key file from: {}", key_filename);
  log::debug!("Loaded SSL certificate chain file from: {}", cert_filename);

  Ok(config)
}
