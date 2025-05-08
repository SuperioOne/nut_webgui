use askama::Template;
use axum::{http::HeaderMap, response::Response};
use axum_core::response::IntoResponse;

#[derive(Template)]
#[template(path = "not_found/+page.html")]
struct NotFound<'a> {
  title: &'a str,
  url_prefix: &'a str,
}

pub async fn get(headers: HeaderMap) -> Response {
  let script_name = headers
    .get("x-script-name")
    .and_then(|v| v.to_str().ok())
    .unwrap_or("");
  let template = NotFound {
    title: "Not Found",
    url_prefix: script_name,
  };
  template.into_response()
}
