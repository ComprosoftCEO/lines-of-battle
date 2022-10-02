use actix::Addr;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws::WsResponseBuilder;
use std::sync::mpsc::Sender;
use uuid::Uuid;

use crate::actors::mediator_messages::{Connect, ConnectResponse};
use crate::actors::{GameMediatorActor, WebsocketActor};
use crate::errors::{ServiceError, WebsocketError};
use crate::jwt::PlayerWebsocketToken;
use crate::protocol::PlayerAction;
use crate::WS_PROTOCOL;

pub async fn connect_player(
  token: PlayerWebsocketToken,
  mediator: web::Data<Addr<GameMediatorActor>>,
  send_player_actions: web::Data<Sender<(Uuid, PlayerAction)>>,
  req: HttpRequest,
  payload: web::Payload,
) -> Result<HttpResponse, ServiceError> {
  let player_id = token.get_id();
  let player_name = token.get_data().get_name().clone();

  // Start the websocket actor to manage the communication
  log::debug!("Connecting player \"{}\" (ID: {})", player_name, player_id);
  log::debug!("Starting actor to handle websocket communication...");
  let (addr, response) = WsResponseBuilder::new(
    WebsocketActor::new(
      token.into_inner(),
      mediator.as_ref().clone(),
      send_player_actions.as_ref().clone(),
    ),
    &req,
    payload,
  )
  .protocols(&[WS_PROTOCOL])
  .start_with_addr()
  .map_err(|e| ServiceError::WebsocketError(WebsocketError::from(e)))?;

  // Register the actor with the mediator -- might return an error
  log::debug!("Registering actor with the game mediator...");
  let connect_response = mediator
    .send(Connect(player_id, addr.clone()))
    .await
    .map_err(ServiceError::WebsocketMailboxError)?;

  // The message handler will close the actor if there is an error
  addr.do_send(connect_response);
  match connect_response {
    ConnectResponse::Ok(_) => {},
    ConnectResponse::NotRegistered => return Err(ServiceError::NotRegistered(player_id)),
    ConnectResponse::AlreadyConnected => return Err(ServiceError::AlreadyConnected(player_id)),
  }

  // Connection is golden!
  Ok(response)
}
