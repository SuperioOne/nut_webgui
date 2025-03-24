#[macro_export]
macro_rules! htmx_redirect {
  ($c:expr, $u:expr) => {{
    let code: axum::http::StatusCode = $c;
    let headers = axum::http::HeaderMap::new();

    (code, [("HX-Redirect", $u)], headers)
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
