/// Overrides destination field when source field is [Some(T)]
macro_rules! override_opt_field {
  ($field:expr, $value:expr) => {{
    let val = $value;
    if val.is_some() {
      $field = val;
    }
  }};

  ($field:expr,inner_value: $value:expr) => {{
    if let Some(inner) = $value {
      $field = inner;
    }
  }};
}

pub(crate) use override_opt_field;
