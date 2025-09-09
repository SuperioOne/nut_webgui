use rand_chacha::{
  ChaCha20Rng,
  rand_core::{RngCore, SeedableRng},
};
use std::time::Duration;

pub mod access_token;
pub mod password_str;
pub mod permission;
pub mod signed_token;
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Nonce(u64);

impl Nonce {
  fn new() -> Self {
    let mut rng = ChaCha20Rng::from_os_rng();
    let nonce = rng.next_u64();

    Self(nonce)
  }

  #[inline]
  pub fn as_u64(self) -> u64 {
    self.0
  }
}

impl From<u64> for Nonce {
  #[inline]
  fn from(value: u64) -> Self {
    Self(value)
  }
}
