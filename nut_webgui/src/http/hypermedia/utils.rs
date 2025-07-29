use crate::config::ServerConfig;
use askama::Template;
use std::{any::Any, collections::HashMap};

#[macro_export]
macro_rules! htmx_redirect {
  ($c:expr, $u:expr) => {{
    let code: axum::http::StatusCode = $c;
    let headers = axum::http::HeaderMap::new();

    (code, [("HX-Redirect", $u)], headers)
  }};
}

#[macro_export]
macro_rules! htmx_swap {
  ($response:expr, $swap:literal) => {{
    let mut response = $response.into_response();
    let headers = response.headers_mut();
    headers.insert("HX-Reswap", axum::http::HeaderValue::from_static($swap));

    response
  }};
}

pub trait RenderWithConfig: Template + Sized {
  fn render_with_config(self, config: &ServerConfig) -> Result<String, askama::Error>;
}

impl<T> RenderWithConfig for T
where
  T: Template,
{
  fn render_with_config(self, config: &ServerConfig) -> Result<String, askama::Error> {
    // TODO: I don't like this

    let mut values: HashMap<&'static str, Box<dyn Any>> = HashMap::new();

    values.insert("UPSD__POLL_FREQ", Box::new(config.upsd.poll_freq));
    values.insert("UPSD__POLL_INTERVAL", Box::new(config.upsd.poll_interval));
    values.insert(
      "HTTP_SERVER__BASE_PATH",
      Box::new(config.http_server.base_path.as_str().to_owned()),
    );

    if let Some(theme) = &config.default_theme {
      values.insert("DEFAULT_THEME", Box::new(theme.clone()));
    }

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
