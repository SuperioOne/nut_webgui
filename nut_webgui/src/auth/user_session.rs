use super::{BinaryToken, access_token::AccessToken, error::UserSessionError, username::Username};
use core::str::from_utf8;
use tokio_util::bytes::BufMut;

#[derive(Clone)]
pub struct UserSession {
  username: Username,
  access_token: AccessToken,
}

impl UserSession {
  pub fn new(username: Username, access_token: AccessToken) -> Self {
    Self {
      access_token,
      username,
    }
  }

  #[inline]
  pub fn get_username(&self) -> &Username {
    &self.username
  }

  #[inline]
  pub fn as_access_token(&self) -> &AccessToken {
    &self.access_token
  }
}

impl BinaryToken for UserSession {
  type Error = UserSessionError;

  fn as_bytes(&self) -> Vec<u8> {
    let mut bytes = self.access_token.as_bytes();
    bytes.put_slice(self.username.as_ref().as_bytes());

    bytes
  }

  fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
    if bytes.len() < AccessToken::TOKEN_BYTE_LEN {
      return Err(UserSessionError::InvalidLength);
    } else if bytes.len() == AccessToken::TOKEN_BYTE_LEN {
      return Err(UserSessionError::EmptyUsername);
    }

    let access_token = AccessToken::from_bytes(&bytes[0..AccessToken::TOKEN_BYTE_LEN])?;
    let username_str = from_utf8(&bytes[AccessToken::TOKEN_BYTE_LEN..])
      .map_err(|_| UserSessionError::InvalidUtf8)?;

    let username = Username::new(username_str).map_err(|_| UserSessionError::EmptyUsername)?;

    Ok(UserSession {
      username,
      access_token,
    })
  }
}

impl AsRef<AccessToken> for UserSession {
  #[inline]
  fn as_ref(&self) -> &AccessToken {
    self.as_access_token()
  }
}

impl core::ops::Deref for UserSession {
  type Target = AccessToken;

  fn deref(&self) -> &Self::Target {
    self.as_access_token()
  }
}

#[cfg(test)]
mod tests {
  use crate::auth::{
    BinaryToken,
    access_token::AccessToken,
    permission::Permissions,
    user_session::{UserSession, UserSessionError},
    username::Username,
  };
  use std::time::Duration;

  #[test]
  fn from_bytes_empty() {
    let bytes: &[u8] = &[];

    match UserSession::from_bytes(bytes) {
      Err(UserSessionError::InvalidLength) => assert!(true),
      _ => assert!(false),
    }
  }

  #[test]
  fn from_bytes_edge() {
    let bytes = [0; AccessToken::TOKEN_BYTE_LEN - 1];

    match UserSession::from_bytes(&bytes) {
      Err(UserSessionError::InvalidLength) => assert!(true),
      _ => assert!(false),
    }
  }

  #[test]
  fn from_bytes_empty_username() {
    let access_token = AccessToken::builder()
      .with_valid_until(Duration::from_secs(600))
      .with_permissions(Permissions::SETVAR | Permissions::FSD | Permissions::INSTCMD)
      .build();

    let bytes = access_token.as_bytes();

    match UserSession::from_bytes(&bytes) {
      Err(UserSessionError::EmptyUsername) => assert!(true),
      _ => assert!(false),
    }
  }

  #[test]
  fn into_from_bytes_pipe() {
    let username = Username::new("test_user").unwrap();
    let access_token = AccessToken::builder()
      .with_valid_until(Duration::from_secs(600))
      .with_permissions(Permissions::SETVAR | Permissions::FSD | Permissions::INSTCMD)
      .build();

    let session_bytes = UserSession::new(username, access_token).as_bytes();

    let session = match UserSession::from_bytes(&session_bytes) {
      Ok(v) => v,
      Err(_) => {
        return assert!(false, "piping into_byte to from_bytes breaks the format");
      }
    };

    assert!(session.access_token.is_active());
    assert!(session.access_token.has_permission(Permissions::SETVAR));
    assert!(session.access_token.has_permission(Permissions::FSD));
    assert!(session.access_token.has_permission(Permissions::INSTCMD));
    assert_eq!("test_user", session.get_username().as_ref());
  }

  #[test]
  fn invalid_utf8_username() {
    let access_token = AccessToken::builder()
      .with_valid_until(Duration::from_secs(600))
      .with_permissions(Permissions::SETVAR | Permissions::FSD | Permissions::INSTCMD)
      .build();

    let mut bytes = access_token.as_bytes();
    bytes.extend_from_slice(&[0, 159, 146, 150]); // Invalid UTF-8

    match UserSession::from_bytes(&bytes) {
      Err(UserSessionError::InvalidUtf8) => assert!(true),
      _ => assert!(false),
    }
  }
}
