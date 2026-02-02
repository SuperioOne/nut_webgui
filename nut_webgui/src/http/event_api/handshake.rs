use super::message::NutEventMessage;
use crate::{
  auth::{access_token::AccessToken, token_signer::TokenSigner},
  http::event_api::error::HandshakeError,
};
use axum::extract::ws::{Message, WebSocket};
use base64::{Engine as _, prelude::BASE64_STANDARD};
use std::time::{Duration, Instant};
use tokio::time::timeout;

const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(30);

pub struct SessionInfo {
  start: Instant,
  duration: Duration,
}

impl SessionInfo {
  pub fn new(duration: Duration) -> Self {
    Self {
      duration,
      start: Instant::now(),
    }
  }

  pub fn is_expired(&self) -> bool {
    self.start.elapsed() >= self.duration
  }
}

pub async fn auth_handshake(
  socket: &mut WebSocket,
  server_key: &[u8],
) -> Result<SessionInfo, HandshakeError> {
  match timeout(HANDSHAKE_TIMEOUT, inner_auth_handshake(socket, server_key)).await {
    Ok(result) => result,
    Err(_) => Err(HandshakeError::TimedOut),
  }
}

async fn inner_auth_handshake(
  socket: &mut WebSocket,
  server_key: &[u8],
) -> Result<SessionInfo, HandshakeError> {
  socket
    .send(NutEventMessage::WaitingForAuth.try_into()?)
    .await?;

  loop {
    match socket.recv().await {
      Some(message) => match message? {
        Message::Text(utf8_bytes) => {
          let content = utf8_bytes.as_str();

          match content.strip_prefix("LOGIN:") {
            Some(key) => {
              let bytes = BASE64_STANDARD
                .decode(key.trim().as_bytes())
                .map_err(|_| HandshakeError::InvalidAccessToken)?;

              let access_token: AccessToken = TokenSigner::new(server_key)
                .from_bytes(&bytes)
                .map_err(|_| HandshakeError::InvalidAccessToken)?;

              if access_token.is_active() {
                socket.send(NutEventMessage::AuthOk.try_into()?).await?;

                return Ok(SessionInfo::new(access_token.ttl()));
              } else {
                return Err(HandshakeError::ExpiredKey);
              }
            }
            None => return Err(HandshakeError::InvalidHandshakeMessage),
          }
        }
        Message::Binary(_) => return Err(HandshakeError::InvalidHandshakeMessage),
        Message::Ping(_) => continue,
        Message::Pong(_) => continue,
        Message::Close(_) => return Err(HandshakeError::ConnectionClosed),
      },
      None => return Err(HandshakeError::ConnectionClosed),
    }
  }
}
