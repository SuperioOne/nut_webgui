use self::{
  error::SendError,
  handshake::{SessionInfo, auth_handshake},
  message::NutEventMessage,
  message_broadcast::MessagePayload,
};
use crate::{http::event_api::error::HandshakeError, state::ServerState};
use axum::{
  extract::{
    State, WebSocketUpgrade,
    ws::{Message, WebSocket},
  },
  response::Response,
};
use futures::{
  SinkExt, StreamExt,
  stream::{SplitSink, SplitStream},
};
use std::sync::Arc;
use tokio::{
  select,
  sync::broadcast::{Receiver, error::RecvError},
  try_join,
};
use tokio_util::sync::CancellationToken;
use tracing::warn;

mod error;
mod handshake;
mod message;

pub mod message_broadcast;

pub async fn wss_handler(ws: WebSocketUpgrade, State(state): State<Arc<ServerState>>) -> Response {
  let receiver = state.message_broadcast.subscribe();
  ws.on_upgrade(|socket| handler(socket, receiver, None))
}

pub async fn wss_with_auth_handler(
  ws: WebSocketUpgrade,
  State(state): State<Arc<ServerState>>,
) -> Response {
  ws.on_upgrade(|socket| handler_with_auth(socket, state))
}

async fn handler(
  socket: WebSocket,
  listener: Receiver<MessagePayload>,
  session: Option<SessionInfo>,
) {
  let cancellation = CancellationToken::new();
  let (sender, receiver) = socket.split();
  let message_sink = MessageSink::new(sender).set_session(session);
  let s_cancel = cancellation.clone();
  let send = tokio::spawn(socket_send(message_sink, listener, s_cancel));
  let listen = tokio::spawn(socket_recv(receiver, cancellation));

  match try_join!(send, listen) {
    Ok((sink, stream)) => {
      if let Ok(mut socket) = sink.reunite(stream) {
        _ = socket.close().await;
      } else {
        warn!(message = "unexpected split state in web socket connection");
      }
    }
    Err(err) => {
      warn!(message = "failed to join web socket tasks", reason = %err);
    }
  }
}

async fn socket_send(
  mut sender: MessageSink,
  mut listener: Receiver<MessagePayload>,
  cancellation: CancellationToken,
) -> SplitSink<WebSocket, Message> {
  loop {
    select! {
      payload = listener.recv() => {
        match payload {
          Ok(payload) => {
            if let Err(err) = sender.send_payload(payload).await {
              cancellation.cancel();
              warn!(message = "web socket session is closed", reason = %err);

              return match err {
                SendError::SocketError { .. } => sender.end(),
                SendError::SessionExpired => sender.end_with_notify().await,
              };
            }
          }
          Err(RecvError::Lagged(lagged)) => {
            warn!(message = "web socket connection is lagging", lagged_message_count = lagged );
          }
          Err(RecvError::Closed) => {
            cancellation.cancel();
            return sender.end();
          },
        }
      },
      _ = cancellation.cancelled() => {
        return sender.end_with_notify().await;
      }
    }
  }
}

async fn socket_recv(
  mut receiver: SplitStream<WebSocket>,
  cancellation: CancellationToken,
) -> SplitStream<WebSocket> {
  'RECEIVE: loop {
    select! {
      msg = receiver.next() => {
        match msg {
          Some(Ok(Message::Ping(..) | Message::Pong(..))) => continue 'RECEIVE,
          _ => {
            cancellation.cancel();
            break 'RECEIVE;
          }
        }
      }
      _ = cancellation.cancelled() => { break 'RECEIVE; }
    }
  }

  receiver
}

async fn handler_with_auth(mut socket: WebSocket, server_state: Arc<ServerState>) {
  match auth_handshake(&mut socket, server_state.config.server_key.as_ref()).await {
    Ok(session) => {
      let receiver = server_state.message_broadcast.subscribe();
      handler(socket, receiver, Some(session)).await
    }
    Err(err) => {
      match err {
        HandshakeError::ConnectionClosed => {
          // skip, do not send any error message
        }
        _ => {
          let msg = NutEventMessage::HandshakeError { message: &err };

          if let Ok(msg) = msg.try_into() {
            _ = socket.send(msg).await;
          }
        }
      };

      _ = socket.close().await;
    }
  }
}

/// 'Smoll' sink wrapper to provide utility functions with optional session check.
struct MessageSink {
  sink: SplitSink<WebSocket, Message>,
  session_info: Option<SessionInfo>,
}

impl MessageSink {
  pub const fn new(sink: SplitSink<WebSocket, Message>) -> Self {
    Self {
      sink,
      session_info: None,
    }
  }

  #[inline]
  pub const fn set_session(mut self, session: Option<SessionInfo>) -> Self {
    self.session_info = session;
    self
  }

  pub fn end(self) -> SplitSink<WebSocket, Message> {
    self.sink
  }

  pub async fn send_payload(&mut self, payload: MessagePayload) -> Result<(), SendError> {
    if self.session_info.as_ref().is_some_and(|v| v.is_expired()) {
      Err(SendError::SessionExpired)
    } else {
      for value in payload.as_ref() {
        let message = Message::text(value);
        self.sink.feed(message).await?;
      }

      self.sink.flush().await?;

      Ok(())
    }
  }

  pub async fn end_with_notify(mut self) -> SplitSink<WebSocket, Message> {
    if let Ok(msg) = NutEventMessage::SessionEnded.try_into() {
      _ = self.sink.send(msg).await;
    }

    self.sink
  }
}
