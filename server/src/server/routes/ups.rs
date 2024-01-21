use crate::server::routes::ups::instcmd_response::instcmd_response;
use crate::server::ServerState;
use axum::extract::{Path, Query, State};
use axum::Form;
use axum_core::response::IntoResponse;
use serde::Deserialize;
use std::sync::Arc;

mod html_response;
mod instcmd_response;

#[derive(Deserialize)]
pub struct UpsQuery {
  section: Option<String>,
}

#[derive(Deserialize)]
pub struct CommandRequest {
  command: String,
}

pub async fn handler(
  path: Path<String>,
  query: Query<UpsQuery>,
  state: State<Arc<ServerState>>,
) -> impl IntoResponse {
  if let Some(section) = &query.section {
    match section.as_ref() {
      "var" => {
        return html_response::partial_page_variables(path, state)
          .await
          .into_response()
      }
      "info" => {
        return html_response::partial_page_info(path, state)
          .await
          .into_response()
      }
      _ => {}
    };
  }

  html_response::page_response(path, state)
    .await
    .into_response()
}

pub async fn handler_command(
  state: State<Arc<ServerState>>,
  path: Path<String>,
  form: Form<CommandRequest>,
) -> impl IntoResponse {
  instcmd_response(state, path, form).await
}
