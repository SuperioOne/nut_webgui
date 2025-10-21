use crate::{
  auth::{permission::Permissions, user_session::UserSession},
  config::ServerConfig,
};
use askama::Template;
use std::{any::Any, borrow::Cow, collections::HashMap};

#[macro_export]
macro_rules! htmx_swap {
  ($response:expr, $swap:literal) => {{
    let mut response = $response.into_response();
    let headers = response.headers_mut();
    headers.insert("HX-Reswap", axum::http::HeaderValue::from_static($swap));

    response
  }};
}

macro_rules! redirect_not_found {
  ($state:expr) => {
    axum::response::Redirect::permanent(&format!(
      "{}/not-found",
      $state.config.http_server.base_path
    ))
    .into_response()
  };
}

pub(super) use redirect_not_found;

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

pub trait RenderWithConfig: Template + Sized {
  fn render_with_config(
    self,
    config: &ServerConfig,
    user: Option<&UserSession>,
  ) -> Result<String, askama::Error>;
}

impl<T> RenderWithConfig for T
where
  T: Template,
{
  fn render_with_config(
    self,
    config: &ServerConfig,
    user: Option<&UserSession>,
  ) -> Result<String, askama::Error> {
    let mut permissions = Permissions::all();
    let mut values: HashMap<&'static str, &dyn Any> = HashMap::new();

    values.insert("UPSD__POLL_FREQ", &30_u64);
    values.insert("UPSD__POLL_INTERVAL", &3_u64);
    values.insert("HTTP_SERVER__BASE_PATH", &config.http_server.base_path);

    if let Some(theme) = &config.default_theme {
      values.insert("DEFAULT_THEME", theme);
    }

    if let Some(profile) = user {
      values.insert("USER_NAME", profile.get_username());
      permissions = profile.get_permissions();
    }

    values.insert("USER_PERMISSION", &permissions);
    self.render_with_values(&values)
  }
}

#[derive(Debug)]
pub struct AppDetails {
  pub version: &'static str,
}

pub const fn get_app_info() -> AppDetails {
  AppDetails {
    version: env!("CARGO_PKG_VERSION"),
  }
}
