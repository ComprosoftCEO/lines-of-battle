//
// Actor that broadcasts the websocket notifications
//
use actix::fut::wrap_future;
use actix::prelude::*;
use actix_http::ws::{CloseCode, CloseReason};
use actix_web_actors::ws;
use serde::Serialize;
use uuid::Uuid;

use crate::actors::{mediator_messages::*, shared_messages::*, GameMediatorActor};
use crate::errors::{ServiceError, WebsocketError};
use crate::game::ServerState;
use crate::protocol::{QueryResponse, ToBytestring, ViewerMessage};

/// Actor used for managing the viewer communication
pub struct ViewerActor {
  id: Uuid,
  server_state: ServerState,
  game_mediator: Addr<GameMediatorActor>,
}

impl ViewerActor {
  pub fn new(id: Uuid, game_mediator: Addr<GameMediatorActor>) -> Self {
    Self {
      id,
      game_mediator,
      server_state: ServerState::Registration,
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

  /// Send an error message back to the clinet
  fn send_error(error: impl Into<ServiceError>, ctx: &mut <Self as Actor>::Context) {
    let error = error.into().get_error_response();
    log::warn!("{}", error.get_description());

    Self::send_json(&error, ctx);
  }

  /// Send a fatal error message and stop the actor
  fn fatal_error(&self, error: impl Into<ServiceError>, close_code: CloseCode, ctx: &mut <Self as Actor>::Context) {
    let error = error.into().get_error_response();
    log::error!(
      "Closing viewer {}: {} (Code {:#?})",
      self.id,
      error.get_description(),
      close_code
    );

    Self::send_json(&error, ctx);
    ctx.close(Some(CloseReason::from((close_code, error.get_description().clone()))));
    ctx.stop();
  }
}

///
/// Make ViewerActor into an actor that can run in the background
///
impl Actor for ViewerActor {
  type Context = ws::WebsocketContext<Self>;

  fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
    // Remove all references to this actor
    self.game_mediator.do_send(DisconnectViewer(ctx.address()));
    Running::Stop
  }
}

///
/// Handler for individual websocket messages
///
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ViewerActor {
  fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    log::debug!("Received message: {:#?}", msg);
    let msg: ws::Message = match msg {
      Err(e) => return Self::send_error(WebsocketError::ProtocolError(e), ctx),
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
      ws::Message::Text(text) => match serde_json::from_str::<ViewerMessage>(&text) {
        Err(e) => return Self::send_error(WebsocketError::JSONError(e), ctx),
        Ok(json) => json,
      },

      // Unsupported messages
      ws::Message::Binary(_) => {
        return Self::send_error(WebsocketError::UnsupportedFrameType("Binary".into()), ctx);
      },
      ws::Message::Continuation(_) => {
        return Self::send_error(WebsocketError::UnsupportedFrameType("Continuation".into()), ctx);
      },
    };

    // Handle the JSON messages
    match json {
      ViewerMessage::GetServerState => self.send_current_state(ctx),
      ViewerMessage::GetRegisteredPlayers => self.send_registered_players(ctx),
    }
  }

  fn finished(&mut self, ctx: &mut Self::Context) {
    log::debug!("Websocket stream closed, stopping actor");
    ctx.stop()
  }
}

impl Handler<ConnectViewerResponse> for ViewerActor {
  type Result = ();

  fn handle(&mut self, ConnectViewerResponse(state): ConnectViewerResponse, ctx: &mut Self::Context) -> Self::Result {
    self.server_state = state;

    // Special case: an error state should clse the connection
    if state == ServerState::FatalError {
      self.fatal_error(ServiceError::GameEngineCrash, CloseCode::Error, ctx);
    }
  }
}

impl Handler<GameEngineCrash> for ViewerActor {
  type Result = ();

  fn handle(&mut self, _: GameEngineCrash, ctx: &mut Self::Context) -> Self::Result {
    self.fatal_error(ServiceError::GameEngineCrash, CloseCode::Error, ctx);
  }
}

impl Handler<RegistrationUpdate> for ViewerActor {
  type Result = ();

  fn handle(&mut self, update: RegistrationUpdate, ctx: &mut Self::Context) -> Self::Result {
    ctx.text(update.into_bytestring());
  }
}

impl Handler<GameStarting> for ViewerActor {
  type Result = ();

  fn handle(&mut self, starting: GameStarting, ctx: &mut Self::Context) -> Self::Result {
    self.server_state = ServerState::Initializing;
    ctx.text(starting.into_bytestring())
  }
}

impl Handler<Init> for ViewerActor {
  type Result = ();

  fn handle(&mut self, init: Init, ctx: &mut Self::Context) -> Self::Result {
    self.server_state = ServerState::Running;
    ctx.text(init.into_bytestring())
  }
}

impl Handler<NextState> for ViewerActor {
  type Result = ();

  fn handle(&mut self, state: NextState, ctx: &mut Self::Context) -> Self::Result {
    ctx.text(state.into_bytestring())
  }
}

impl Handler<PlayerKilled> for ViewerActor {
  type Result = ();

  fn handle(&mut self, player_killed: PlayerKilled, ctx: &mut Self::Context) -> Self::Result {
    ctx.text(player_killed.into_bytestring())
  }
}

impl Handler<GameEnded> for ViewerActor {
  type Result = ();

  fn handle(&mut self, game_ended: GameEnded, ctx: &mut Self::Context) -> Self::Result {
    self.server_state = ServerState::Registration;
    ctx.text(game_ended.into_bytestring())
  }
}

impl ViewerActor {
  fn send_current_state(&self, ctx: &mut <Self as Actor>::Context) {
    Self::send_json(
      &QueryResponse::ServerState {
        state: self.server_state,
      },
      ctx,
    );
  }

  fn send_registered_players(&self, ctx: &mut <Self as Actor>::Context) {
    // Spawn a future to process the request
    ctx.spawn(
      wrap_future::<_, Self>(self.game_mediator.send(GetRegisteredPlayers)).map(|result, _this, ctx| match result {
        Ok(registered) => Self::send_json(
          &QueryResponse::RegisteredPlayers {
            players: registered.players,
            player_order: registered.player_order,
          },
          ctx,
        ),
        Err(e) => Self::send_error(ServiceError::WebsocketMailboxError(e), ctx),
      }),
    );
  }
}
