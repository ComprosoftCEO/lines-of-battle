use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::{offset::Utc, Duration};
use jsonwebtoken::{decode, encode, Algorithm, EncodingKey, Header, Validation};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use uuid::Uuid;

use super::JWTSecret;
use crate::errors::ServiceError;
use crate::jwt::{Audience, JWT_ISSUER};

/// JSON Web Token used for user authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JWTToken<A: Audience, T> {
  // Reserved Claims
  iss: String, // Issuer
  sub: Uuid,   // Subject (whom token refers to)
  aud: String, // Audience (whom the token is intended for)
  iat: i64,    // Issued at (as UTC timestamp)
  exp: i64,    // Expiration time (as UTC timestamp)

  // Public and private claims
  #[serde(flatten)]
  user_data: T,

  #[serde(skip)]
  _aud: PhantomData<A>,
}

impl<A: Audience, T> JWTToken<A, T> {
  pub fn new(subject: Uuid, expires_in: Duration, user_data: T) -> Self {
    let now = Utc::now();
    let expiration = now + expires_in;

    Self {
      iss: JWT_ISSUER.to_string(),
      sub: subject,
      aud: A::get_name(),
      iat: now.timestamp(),
      exp: expiration.timestamp(),

      user_data,

      _aud: PhantomData,
    }
  }

  pub fn get_id(&self) -> Uuid {
    self.sub
  }

  pub fn get_data(&self) -> &T {
    &self.user_data
  }

  pub fn into_data(self) -> T {
    self.user_data
  }
}

impl<A, T> JWTToken<A, T>
where
  A: Audience,
  T: Serialize + DeserializeOwned,
{
  /// Encode the JSON Web Token into a string
  pub fn encode(&self, key: &EncodingKey) -> Result<String, jsonwebtoken::errors::Error> {
    Ok(encode(&Header::new(Algorithm::HS256), self, key)?)
  }
}

//
// Get the JSON Web Token from the request
//
impl<A, T> FromRequest for JWTToken<A, T>
where
  A: Audience,
  T: Serialize + DeserializeOwned,
{
  type Error = ServiceError;
  type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

  fn from_request(req: &HttpRequest, _pl: &mut Payload) -> Self::Future {
    let req = req.clone();
    Box::pin(async move {
      // Extract the JWT from the header and the encryption key from the app data
      let bearer_token = BearerAuth::extract(&req).await?;
      let jwt_public_key = req.app_data::<web::Data<JWTSecret>>().expect("JWTSecret should be set");

      // Validation parameters,
      let validation = Validation {
        algorithms: vec![Algorithm::HS256],
        validate_exp: true,
        leeway: 15,
        aud: Some(A::accepts()),
        iss: Some(JWT_ISSUER.into()),
        ..Default::default()
      };

      // Decode and validate the JWT
      let token_data = decode::<Self>(bearer_token.token(), &jwt_public_key.get_decoding_key(), &validation)?;
      Ok(token_data.claims)
    })
  }
}
