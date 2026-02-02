use crate::auth::error::AccessTokenError;
use serde::Serialize;
use tokio::time::error::Elapsed;

#[derive(Debug)]
pub enum HandshakeError {
  ConnectionClosed,
  ExpiredKey,
  InvalidAccessToken,
  InvalidHandshakeMessage,
  SerializeError { inner: serde_json::error::Error },
  SocketError { inner: axum::Error },
  TimedOut,
}

pub enum SendError {
  SessionExpired,
  SocketError { inner: axum::Error },
}

impl From<Elapsed> for HandshakeError {
  #[inline]
  fn from(_: Elapsed) -> Self {
    Self::TimedOut
  }
}

impl From<axum::Error> for HandshakeError {
  #[inline]
  fn from(value: axum::Error) -> Self {
    Self::SocketError { inner: value }
  }
}

impl From<serde_json::error::Error> for HandshakeError {
  #[inline]
  fn from(value: serde_json::error::Error) -> Self {
    Self::SerializeError { inner: value }
  }
}

impl From<axum::Error> for SendError {
  #[inline]
  fn from(value: axum::Error) -> Self {
    Self::SocketError { inner: value }
  }
}

impl From<AccessTokenError> for HandshakeError {
  #[inline]
  fn from(_: AccessTokenError) -> Self {
    Self::InvalidAccessToken
  }
}

impl core::fmt::Display for HandshakeError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      HandshakeError::ConnectionClosed => f.write_str("connection closed unexpectedly"),
      HandshakeError::ExpiredKey => f.write_str("key has expired"),
      HandshakeError::InvalidAccessToken => f.write_str("invalid access token"),
      HandshakeError::InvalidHandshakeMessage => f.write_str("invalid handshake message received"),
      HandshakeError::SerializeError { inner } => inner.fmt(f),
      HandshakeError::SocketError { inner } => inner.fmt(f),
      HandshakeError::TimedOut => f.write_str("handshake timed out"),
    }
  }
}

impl core::fmt::Display for SendError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      SendError::SocketError { inner } => inner.fmt(f),
      SendError::SessionExpired => f.write_str("auth key is expired"),
    }
  }
}

impl Serialize for HandshakeError {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}
