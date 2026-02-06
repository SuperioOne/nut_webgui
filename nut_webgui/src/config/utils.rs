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

pub fn rand_server_key_256bit() -> Box<[u8]> {
  let mut bytes = [0u8; 32];
  getrandom::fill(&mut bytes).expect("system's random fill has failed at server key generation");
  Box::from(bytes)
}
