use std::time::Duration;

pub mod access_token;
pub mod error;
pub mod nonce;
pub mod password_str;
pub mod permission;
pub mod token_signer;
pub mod user_session;
pub mod user_store;
pub mod username;

pub const AUTH_COOKIE_NAME: &str = "_nutwg";
pub const AUTH_COOKIE_DURATION: Duration = Duration::from_secs(3600 * 24 * 30);
pub const AUTH_COOKIE_RENEW: Duration = Duration::from_secs(3600 * 24 * 7);

pub trait BinaryToken: Sized {
  type Error;

  fn as_bytes(&self) -> Vec<u8>;

  fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Error>;
}
