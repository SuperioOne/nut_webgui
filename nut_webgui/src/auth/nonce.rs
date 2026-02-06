#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Nonce(u64);

impl Nonce {
  pub fn new() -> Self {
    let value = getrandom::u64().expect("system rand function has failed!");
    Self(value)
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
