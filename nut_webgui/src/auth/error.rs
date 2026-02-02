use super::BinaryToken;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmptyUsername;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionError {
  InvalidLogin,
  InvalidOutputLength,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmptyPasswordStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserSessionError {
  InvalidAccessToken(AccessTokenError),
  InvalidLength,
  InvalidUtf8,
  EmptyUsername,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessTokenError {
  InvalidVersion,
  InvalidLength,
  InvalidTimestamps,
}

pub enum SignatureError<T>
where
  T: BinaryToken,
{
  InvalidLength,
  InvalidSignature,
  TokenError(T::Error),
}

impl core::fmt::Display for EmptyUsername {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str("username cannot be empty")
  }
}

impl core::fmt::Display for SessionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      SessionError::InvalidLogin => f.write_str("username is not present in user list"),
      SessionError::InvalidOutputLength => f.write_str("hashing output length is invalid"),
    }
  }
}

impl core::fmt::Display for EmptyPasswordStr {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("password string cannot be empty or whitespace only")
  }
}

impl core::fmt::Display for UserSessionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      UserSessionError::InvalidAccessToken(err) => err.fmt(f),
      UserSessionError::InvalidLength => f.write_str("token length is invalid"),
      UserSessionError::InvalidUtf8 => f.write_str("username string is not a valid utf-8 string"),
      UserSessionError::EmptyUsername => f.write_str("username is empty"),
    }
  }
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

impl From<AccessTokenError> for UserSessionError {
  #[inline]
  fn from(value: AccessTokenError) -> Self {
    Self::InvalidAccessToken(value)
  }
}

impl core::error::Error for AccessTokenError {}
impl core::error::Error for UserSessionError {}
impl core::error::Error for EmptyPasswordStr {}
impl core::error::Error for SessionError {}
impl core::error::Error for EmptyUsername {}
