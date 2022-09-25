//
// Actor that broadcasts the websocket notifications
//
use actix::prelude::*;
use actix_http::ws::{CloseCode, CloseReason};
use actix_http::StatusCode;
use actix_web_actors::ws;
use serde::Serialize;
use uuid::Uuid;

use crate::actors::{message_types::*, GameMediatorActor};
use crate::errors::{ErrorResponse, GlobalErrorCode};

/// Actor used for managing the websocket communication
pub struct WebsocketActor {
  player_id: Uuid,
  game_mediator: Addr<GameMediatorActor>,
}

impl WebsocketActor {
  pub fn new(player_id: Uuid, game_mediator: Addr<GameMediatorActor>) -> Self {
    Self {
      player_id,
      game_mediator,
    }
  }

  /// Send a JSON response back to the client, handling any serialization errors
  fn send_json<T>(data: &T, ctx: &mut <Self as Actor>::Context)
  where
    T: ?Sized + Serialize,
  {
    match serde_json::to_string(data) {
      Ok(json) => ctx.text(json),
      Err(e) => log::error!("Failed to serialize JSON data: {}", e),
    }
  }
}

/// Close the websocket connection due to an error
#[derive(Message)]
#[rtype(result = "()")]
struct FatalErrorClose(CloseCode, Option<String>);

/// Send back an error response, but keep the websocket open
#[derive(Message)]
#[rtype(result = "()")]
struct NonFatalError {
  message: String,
  developer_notes: Option<String>,
}

impl From<CloseCode> for FatalErrorClose {
  fn from(code: CloseCode) -> Self {
    Self(code, None)
  }
}

impl<T> From<(CloseCode, T)> for FatalErrorClose
where
  T: Into<String>,
{
  fn from((code, description): (CloseCode, T)) -> Self {
    Self(code, Some(description.into()))
  }
}

///
/// Make WebsocketActor into an actor that can run in the background
///
impl Actor for WebsocketActor {
  type Context = ws::WebsocketContext<Self>;

  fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
    // Remove all references to this actor
    self.game_mediator.do_send(Disconnect(self.player_id));
    Running::Stop
  }
}

///
/// Handler for individual websocket messages
///
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebsocketActor {
  fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    let self_addr = ctx.address();

    log::debug!("Received message: {:#?}", msg);
    let msg: ws::Message = match msg {
      Err(e) => return self_addr.do_send(FatalErrorClose::from((CloseCode::Error, format!("{}", e)))),
      Ok(msg) => msg,
    };

    // Parse as a JSON string
    let json = match msg {
      // Basic messages
      ws::Message::Nop => return,
      ws::Message::Ping(msg) => return ctx.pong(&msg),
      ws::Message::Pong(_) => return,
      ws::Message::Close(reason) => {
        log::info!("Received close message, closing... ({:#?})", reason);
        ctx.close(reason);
        return ctx.stop();
      },

      // Parse JSON message
      ws::Message::Text(text) => match serde_json::from_str::<serde_json::Value>(&text) {
        Err(e) => {
          return self_addr.do_send(NonFatalError {
            message: "Invalid JSON data".into(),
            developer_notes: Some(format!("{}", e)),
          })
        },
        Ok(json) => json,
      },

      // Unsupported messages
      ws::Message::Binary(_) => {
        return self_addr.do_send(NonFatalError {
          message: "Unsupported Frame: Binary Data".into(),
          developer_notes: None,
        })
      },
      ws::Message::Continuation(_) => {
        return self_addr.do_send(NonFatalError {
          message: "Unsupported Frame: Continuation".into(),
          developer_notes: None,
        })
      },
    };

    Self::send_json(&json, ctx);
  }

  fn finished(&mut self, ctx: &mut Self::Context) {
    log::debug!("Websocket stream closed, stopping actor");
    ctx.stop()
  }
}

//
// Handle websocket errors
//
impl Handler<FatalErrorClose> for WebsocketActor {
  type Result = ();

  fn handle(&mut self, FatalErrorClose(code, description): FatalErrorClose, ctx: &mut Self::Context) -> Self::Result {
    if let Some(ref description) = description {
      log::error!("Closing websocket: {} (Code {:#?})", description, code);
    } else {
      log::error!("Closing websocket: code {:#?}", code);
    }

    ctx.close(Some(CloseReason { code, description }));
    ctx.stop();
  }
}

impl Handler<NonFatalError> for WebsocketActor {
  type Result = ();

  fn handle(&mut self, error: NonFatalError, ctx: &mut Self::Context) -> Self::Result {
    if let Some(ref developer_notes) = error.developer_notes {
      log::error!("{}: {}", error.message, developer_notes);
    } else {
      log::error!("{}", error.message);
    }

    Self::send_json(
      &ErrorResponse::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        error.message,
        GlobalErrorCode::WebsocketError,
        error.developer_notes.unwrap_or_default(),
      ),
      ctx,
    );
  }
}

impl Handler<ConnectResponse> for WebsocketActor {
  type Result = ();

  fn handle(&mut self, response: ConnectResponse, ctx: &mut Self::Context) -> Self::Result {
    if response != ConnectResponse::Ok {
      ctx.close(Some(CloseCode::Abnormal.into()))
    }
  }
}
