use actix::Addr;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws::WsResponseBuilder;

use crate::actors::mediator_messages::ConnectViewer;
use crate::actors::{GameMediatorActor, ViewerActor};
use crate::errors::{ServiceError, WebsocketError};
use crate::jwt::ViewerWebsocketToken;
use crate::WS_PROTOCOL;

pub async fn connect_viewer(
  token: ViewerWebsocketToken,
  mediator: web::Data<Addr<GameMediatorActor>>,
  req: HttpRequest,
  payload: web::Payload,
) -> Result<HttpResponse, ServiceError> {
  let viewer_id = token.get_id();

  // Start the websocket actor to manage the communication
  log::debug!("Connecting viewer {}", viewer_id);
  log::debug!("Starting actor to handle websocket communication...");
  let (addr, response) = WsResponseBuilder::new(ViewerActor::new(viewer_id, mediator.as_ref().clone()), &req, payload)
    .protocols(&[WS_PROTOCOL])
    .start_with_addr()
    .map_err(|e| ServiceError::WebsocketError(WebsocketError::from(e)))?;

  // Register the actor with the mediator -- might return an error
  log::debug!("Registering viewer with the game mediator...");
  let connect_response = mediator
    .send(ConnectViewer(addr.clone()))
    .await
    .map_err(ServiceError::WebsocketMailboxError)?;

  // The message handler will close the actor if there is an error
  addr.do_send(connect_response);

  // Connection is golden!
  Ok(response)
}
