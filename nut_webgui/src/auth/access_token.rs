use crate::auth::{BinaryToken, Nonce, permission::Permissions};
use chrono::Utc;
use core::time::Duration;
use tokio_util::bytes::BufMut;

/// Simple binary formatted access token payload.
///
/// ## Format:
/// ┌─────────┬─────────────┬───────────┬─────────────┬───────────┐
/// │ version │ permissions │ issued_at │ valid_until │   nonce   │
/// │ 1-byte  │   1-byte    │ LE 8-byte │  LE 8-byte  │ LE 8-byte │
/// └─────────┴─────────────┴───────────┴─────────────┴───────────┘
/// * LE          : Little-Endian byte order.
/// * version     : Version number. (Just in case for backward compability)
/// * permissions : Bit flags defined at [Permissions]
/// * issued_at   : Issue datetime in Unix timestamp (seconds)
/// * valid_until : Token expiration datetime in Unix timestamp (seconds)
/// * nonce       : Cryptographic nonce
#[derive(Clone)]
pub struct AccessToken {
  permissions: Permissions,
  valid_until: u64,
  issued_at: u64,
  nonce: Nonce,
}

pub struct AccessTokenBuilder {
  valid_until: Duration,
  permissions: Permissions,
}

impl AccessTokenBuilder {
  #[inline]
  pub const fn new() -> Self {
    Self {
      permissions: Permissions::READONLY,
      valid_until: Duration::from_secs(60 * 15),
    }
  }

  #[inline]
  pub const fn with_valid_until(mut self, duration: Duration) -> Self {
    self.valid_until = duration;
    self
  }

  #[inline]
  pub const fn with_permissions(mut self, permissions: Permissions) -> Self {
    self.permissions = permissions;
    self
  }

  pub fn build(self) -> AccessToken {
    let now = Utc::now().timestamp() as u64;

    AccessToken {
      permissions: self.permissions,
      nonce: Nonce::new(),
      issued_at: now,
      valid_until: now.saturating_add(self.valid_until.as_secs()),
    }
  }
}

impl AccessToken {
  pub const TOKEN_BYTE_LEN: usize = 26;
  pub const VERSION: u8 = 1;

  #[inline]
  pub const fn builder() -> AccessTokenBuilder {
    AccessTokenBuilder::new()
  }

  pub fn is_active(&self) -> bool {
    let now = Utc::now().timestamp() as u64;
    now >= self.issued_at && now < self.valid_until
  }

  #[inline]
  pub fn ttl(&self) -> Duration {
    let now = Utc::now().timestamp() as u64;
    let remaining = self.valid_until.saturating_sub(now);
    Duration::from_secs(remaining)
  }

  #[inline]
  pub fn has_permission(&self, permissions: Permissions) -> bool {
    self.permissions.has(permissions)
  }

  #[inline]
  pub fn get_permissions(&self) -> Permissions {
    self.permissions
  }
}

impl BinaryToken for AccessToken {
  type Error = AccessTokenError;

  fn as_bytes(&self) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(Self::TOKEN_BYTE_LEN);

    bytes.put_u8(Self::VERSION);
    bytes.put_u8(self.permissions.as_u8());
    bytes.put_u64_le(self.issued_at);
    bytes.put_u64_le(self.valid_until);
    bytes.put_u64_le(self.nonce.as_u64());

    bytes
  }

  fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
    if bytes.len() != Self::TOKEN_BYTE_LEN {
      return Err(AccessTokenError::InvalidLength);
    }

    let version = bytes[0];
    let permissions_val = bytes[1];
    let issued_at = u64::from_le_bytes(bytes[2..10].try_into().expect("invalid issue_at slice"));
    let valid_until =
      u64::from_le_bytes(bytes[10..18].try_into().expect("invalid valid_until slice"));
    let nonce_val = u64::from_le_bytes(bytes[18..26].try_into().expect("invalid nonce_val slice"));

    if version != Self::VERSION {
      return Err(AccessTokenError::InvalidVersion);
    }

    if issued_at > valid_until {
      return Err(AccessTokenError::InvalidTimestamps);
    }

    Ok(AccessToken {
      permissions: Permissions::from(permissions_val),
      issued_at,
      valid_until,
      nonce: Nonce::from(nonce_val),
    })
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessTokenError {
  InvalidVersion,
  InvalidLength,
  InvalidTimestamps,
}

impl core::fmt::Display for AccessTokenError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      AccessTokenError::InvalidVersion => f.write_str("api token version is invalid"),
      AccessTokenError::InvalidLength => f.write_str("token length is invalid"),
      AccessTokenError::InvalidTimestamps => {
        f.write_str("token's issued_at and valid_until dates are not valid")
      }
    }
  }
}

impl core::error::Error for AccessTokenError {}

#[cfg(test)]
mod tests {
  use crate::auth::{
    BinaryToken,
    access_token::{AccessToken, AccessTokenError},
    permission::Permissions,
  };
  use std::time::Duration;
  use tokio_util::bytes::BufMut;

  #[test]
  fn empty_input() {
    let bytes: &[u8] = &[];

    match AccessToken::from_bytes(bytes) {
      Err(AccessTokenError::InvalidLength) => assert!(true),
      _ => assert!(false),
    }
  }

  #[test]
  fn token_length_less_than_expected() {
    let bytes = [0; AccessToken::TOKEN_BYTE_LEN - 1];

    match AccessToken::from_bytes(&bytes) {
      Err(AccessTokenError::InvalidLength) => assert!(true),
      _ => assert!(false),
    }
  }

  #[test]
  fn token_length_more_than_expected() {
    let bytes = [0; AccessToken::TOKEN_BYTE_LEN + 1];

    match AccessToken::from_bytes(&bytes) {
      Err(AccessTokenError::InvalidLength) => assert!(true),
      _ => assert!(false),
    }
  }

  #[test]
  fn invalid_timestamps() {
    let mut bytes = Vec::new();

    bytes.put_u8(1);
    bytes.put_u8(5);
    bytes.put_u64_le(20423);
    bytes.put_u64_le(10423);
    bytes.put_u64_le(99999);

    match AccessToken::from_bytes(&bytes) {
      Err(AccessTokenError::InvalidTimestamps) => assert!(true),
      _ => assert!(false),
    }
  }

  #[test]
  fn invalid_version() {
    let mut bytes = Vec::new();

    bytes.put_u8(2);
    bytes.put_u8(5);
    bytes.put_u64_le(20423);
    bytes.put_u64_le(10423);
    bytes.put_u64_le(99999);

    match AccessToken::from_bytes(&bytes) {
      Err(AccessTokenError::InvalidVersion) => assert!(true),
      _ => assert!(false),
    }
  }

  #[test]
  fn into_from_bytes_pipe() {
    let permissions = Permissions::SETVAR | Permissions::INSTCMD;

    let token = AccessToken::builder()
      .with_permissions(permissions)
      .with_valid_until(Duration::from_secs(3600))
      .build();

    let valid_until = token.valid_until;
    let issued_at = token.issued_at;

    let bytes = token.as_bytes();
    let new_session = AccessToken::from_bytes(&bytes).unwrap();

    assert_eq!(permissions, new_session.permissions);
    assert_eq!(valid_until, new_session.valid_until);
    assert_eq!(issued_at, new_session.issued_at);

    let new_bytes = new_session.as_bytes();
    assert_eq!(bytes, new_bytes);
  }
}
