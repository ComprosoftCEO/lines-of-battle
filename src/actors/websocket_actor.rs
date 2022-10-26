//
// Actor that broadcasts the websocket notifications
//
use actix::fut::wrap_future;
use actix::prelude::*;
use actix_http::ws::{CloseCode, CloseReason};
use actix_web_actors::ws;
use serde::Serialize;
use std::sync::mpsc::Sender;
use uuid::Uuid;

use crate::actors::{mediator_messages::*, shared_messages::*, websocket_messages::*, GameMediatorActor};
use crate::errors::{ServiceError, WebsocketError};
use crate::game::ServerState;
use crate::jwt::{JWTPlayerData, PlayerToken};
use crate::protocol::{PlayerAction, QueryResponse, ToBytestring, WebsocketMessage};

/// Actor used for managing the websocket communication
pub struct WebsocketActor {
  player_id: Uuid,
  player_data: JWTPlayerData,
  game_mediator: Addr<GameMediatorActor>,
  send_player_action: Sender<(Uuid, PlayerAction)>,

  server_state: ServerState,
  action_sent: bool,
  player_killed: bool,
}

impl WebsocketActor {
  pub fn new(
    player_token: PlayerToken,
    game_mediator: Addr<GameMediatorActor>,
    send_player_action: Sender<(Uuid, PlayerAction)>,
  ) -> Self {
    Self {
      player_id: player_token.get_id(),
      player_data: player_token.into_data(),
      game_mediator,
      send_player_action,

      server_state: ServerState::Registration,
      action_sent: false,
      player_killed: false,
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
  fn fatal_error(error: impl Into<ServiceError>, close_code: CloseCode, ctx: &mut <Self as Actor>::Context) {
    let error = error.into().get_error_response();
    log::error!(
      "Closing websocket: {} (Code {:#?})",
      error.get_description(),
      close_code
    );

    Self::send_json(&error, ctx);
    ctx.close(Some(CloseReason::from((close_code, error.get_description().clone()))));
    ctx.stop();
  }
}

///
/// Make WebsocketActor into an actor that can run in the background
///
impl Actor for WebsocketActor {
  type Context = ws::WebsocketContext<Self>;

  fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
    // Remove all references to this actor
    self.game_mediator.do_send(Disconnect(self.player_id, ctx.address()));
    Running::Stop
  }
}

///
/// Handler for individual websocket messages
///
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebsocketActor {
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
      ws::Message::Text(text) => match serde_json::from_str::<WebsocketMessage>(&text) {
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

    match json {
      WebsocketMessage::Register => self.register(ctx),
      WebsocketMessage::Unregister => self.unregister(ctx),
      WebsocketMessage::GetServerState => self.send_server_state(ctx),
      WebsocketMessage::Move(action) => self.do_action(action.transpose(), ctx),
      WebsocketMessage::Attack(action) => self.do_action(action.transpose(), ctx),
      WebsocketMessage::DropWeapon(action) => self.do_action(action.transpose(), ctx),
    }
  }

  fn finished(&mut self, ctx: &mut Self::Context) {
    log::debug!("Websocket stream closed, stopping actor");
    ctx.stop()
  }
}

impl Handler<ConnectResponse> for WebsocketActor {
  type Result = ();

  fn handle(&mut self, response: ConnectResponse, ctx: &mut Self::Context) -> Self::Result {
    match response {
      ConnectResponse::Ok(state) => {
        self.server_state = state;
        if self.server_state == ServerState::FatalError {
          Self::fatal_error(ServiceError::GameEngineCrash, CloseCode::Error, ctx);
        }
      },
      _ => ctx.close(Some(CloseCode::Abnormal.into())),
    }
  }
}

impl Handler<GameEngineCrash> for WebsocketActor {
  type Result = ();

  fn handle(&mut self, _: GameEngineCrash, ctx: &mut Self::Context) -> Self::Result {
    Self::fatal_error(ServiceError::GameEngineCrash, CloseCode::Error, ctx);
  }
}

impl Handler<RegistrationUpdate> for WebsocketActor {
  type Result = ();

  fn handle(&mut self, update: RegistrationUpdate, ctx: &mut Self::Context) -> Self::Result {
    ctx.text(update.into_bytestring());
  }
}

impl Handler<KickUnregisteredPlayer> for WebsocketActor {
  type Result = ();

  fn handle(&mut self, _: KickUnregisteredPlayer, ctx: &mut Self::Context) -> Self::Result {
    Self::fatal_error(ServiceError::NotRegistered(self.player_id), CloseCode::Error, ctx);
  }
}

impl Handler<GameStarting> for WebsocketActor {
  type Result = ();

  fn handle(&mut self, starting: GameStarting, ctx: &mut Self::Context) -> Self::Result {
    self.server_state = ServerState::Initializing;
    ctx.text(starting.into_bytestring())
  }
}

impl Handler<Init> for WebsocketActor {
  type Result = ();

  fn handle(&mut self, init: Init, ctx: &mut Self::Context) -> Self::Result {
    self.server_state = ServerState::Running;
    self.action_sent = false;
    self.player_killed = false;

    ctx.text(init.into_bytestring())
  }
}

impl Handler<NextState> for WebsocketActor {
  type Result = ();

  fn handle(&mut self, state: NextState, ctx: &mut Self::Context) -> Self::Result {
    self.action_sent = false;
    ctx.text(state.into_bytestring())
  }
}

impl Handler<PlayerKilled> for WebsocketActor {
  type Result = ();

  fn handle(&mut self, player_killed: PlayerKilled, ctx: &mut Self::Context) -> Self::Result {
    if player_killed.get_player_id() == self.player_id {
      self.player_killed = true;
    }
    ctx.text(player_killed.into_bytestring())
  }
}

impl Handler<GameEnded> for WebsocketActor {
  type Result = ();

  fn handle(&mut self, game_ended: GameEnded, ctx: &mut Self::Context) -> Self::Result {
    self.server_state = ServerState::Registration;
    ctx.text(game_ended.into_bytestring())
  }
}

impl WebsocketActor {
  fn register(&self, ctx: &mut <Self as Actor>::Context) {
    // Spawn a future to process the request
    ctx.spawn(
      wrap_future::<_, Self>(self.game_mediator.send(Register {
        id: self.player_id,
        data: self.player_data.clone(),
      }))
      .map(|result, this, ctx| match result {
        Ok(RegisterResponse::Success) => {},
        Ok(RegisterResponse::GameAlreadyStarted) => Self::send_error(
          ServiceError::FailedToRegister(this.player_id, "game already started".into()),
          ctx,
        ),
        Ok(RegisterResponse::TooManyRegistered { max_allowed }) => Self::send_error(
          ServiceError::FailedToRegister(
            this.player_id,
            format!("too many players registered ({} maximum allowed)", max_allowed),
          ),
          ctx,
        ),
        Err(e) => Self::send_error(ServiceError::WebsocketMailboxError(e), ctx),
      }),
    );
  }

  fn unregister(&self, ctx: &mut <Self as Actor>::Context) {
    // Spawn a future to process the request
    ctx.spawn(
      wrap_future::<_, Self>(self.game_mediator.send(Unregister { id: self.player_id })).map(|result, this, ctx| {
        match result {
          Ok(true) => {},
          Ok(false) => Self::send_error(ServiceError::FailedToUnregister(this.player_id), ctx),
          Err(e) => Self::send_error(ServiceError::WebsocketMailboxError(e), ctx),
        }
      }),
    );
  }

  fn send_server_state(&self, ctx: &mut <Self as Actor>::Context) {
    Self::send_json(
      &QueryResponse::ServerState {
        state: self.server_state,
      },
      ctx,
    );
  }

  fn do_action(&mut self, action: PlayerAction, ctx: &mut <Self as Actor>::Context) {
    if self.player_killed {
      return Self::send_error(
        ServiceError::CannotSendAction {
          why: "player has been killed".into(),
        },
        ctx,
      );
    }

    if !self.server_state.can_send_action() {
      return Self::send_error(
        ServiceError::CannotSendAction {
          why: "game has not started yet".into(),
        },
        ctx,
      );
    }

    if self.action_sent {
      return Self::send_error(
        ServiceError::CannotSendAction {
          why: "already sent player action".into(),
        },
        ctx,
      );
    }

    match self.send_player_action.send((self.player_id, action)) {
      Ok(_) => {
        self.action_sent = true;
      },
      Err(_) => {
        return Self::send_error(
          ServiceError::CannotSendAction {
            why: "channel error".into(),
          },
          ctx,
        );
      },
    }
  }
}
