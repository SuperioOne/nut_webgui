use std::borrow::Cow;

pub mod app_info;
pub mod device_entry_impls;
pub mod named_enum;
pub mod render_extra;
pub mod semantic_type;
pub mod topology_graph;
pub mod unit;
pub mod ups_status;

macro_rules! htmx_swap {
  ($response:expr, $swap:literal) => {{
    let mut response = $response.into_response();
    let headers = response.headers_mut();
    headers.insert("HX-Reswap", axum::http::HeaderValue::from_static($swap));

    response
  }};
}

pub(super) use htmx_swap;

pub fn normalize_id(input: &str) -> Cow<'_, str> {
  let first = input.as_bytes().first();

  let input = if first.is_some_and(|v| v.is_ascii_digit()) {
    let mut prefixed = String::new();
    prefixed.push('_');
    prefixed.push_str(input);

    Cow::Owned(prefixed)
  } else {
    Cow::Borrowed(input)
  };

  for ch in input.as_bytes().iter() {
    if !ch.is_ascii_alphanumeric() && *ch != b'_' && *ch != b'-' && *ch != b'.' {
      let escaped = input.replace(
        |input: char| {
          !input.is_ascii_alphanumeric() && input != '.' && input != '_' && input != '-'
        },
        "_",
      );

      return Cow::Owned(escaped);
    }
  }

  input
}
