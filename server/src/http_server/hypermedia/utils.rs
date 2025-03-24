#[macro_export]
macro_rules! htmx_redirect {
  ($c:expr, $u:expr) => {{
    let code: axum::http::StatusCode = $c;
    let uri: &str = $u;
    let headers = axum::http::HeaderMap::new();

    (code, [("HX-Redirect", uri)], headers)
  }};
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
