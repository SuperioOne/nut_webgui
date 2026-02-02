use rand_chacha::{
  ChaCha20Rng,
  rand_core::{RngCore, SeedableRng},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Nonce(u64);

impl Nonce {
  pub fn new() -> Self {
    let mut rng = ChaCha20Rng::from_os_rng();
    let nonce = rng.next_u64();

    Self(nonce)
  }

  #[inline]
  pub const fn as_u64(self) -> u64 {
    self.0
  }
}

impl From<u64> for Nonce {
  #[inline]
  fn from(value: u64) -> Self {
    Self(value)
  }
}
