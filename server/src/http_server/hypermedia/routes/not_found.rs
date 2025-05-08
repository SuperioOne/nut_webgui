use askama::Template;
use axum::{extract::State, response::Response};
use axum_core::response::IntoResponse;

use crate::http_server::ServerState;

#[derive(Template)]
#[template(path = "not_found/+page.html")]
struct NotFound<'a> {
  title: &'a str,
  base_path: &'a str,
}

pub async fn get(State(state): State<ServerState>) -> Response {
  let template = NotFound {
    title: "Not Found",
    base_path: &state.configs.base_path,
  };
  template.into_response()
}
