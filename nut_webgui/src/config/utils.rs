/// Overrides destination field when source field is [Some(T)]
macro_rules! override_opt_field {
  ($field:expr, $value:expr) => {
    if $value.is_some() {
      $field = $value;
    }
  };

  ($field:expr,inner_value: $value:expr) => {{
    if let Some(inner) = $value {
      $field = inner;
    }
  }};
}

pub(crate) use override_opt_field;
use rand_chacha::{
  ChaCha20Rng,
  rand_core::{RngCore, SeedableRng},
};

const CHARSET: &[u8; 64] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ01234567890_";

pub fn rand_server_key_256bit() -> String {
  let mut bytes = vec![0u8; 32];
  let mut rng = ChaCha20Rng::from_os_rng();
  let mut idx: usize = 0;

  for _ in 0..4 {
    let indexs: [u8; 8] = (rng.next_u64() & 0x3F3F3F3F3F3F3F3F).to_le_bytes();

    for key in indexs {
      bytes[idx] = *CHARSET.get(key as usize).unwrap();
      idx += 1;
    }
  }

  unsafe { String::from_utf8_unchecked(bytes) }
}
