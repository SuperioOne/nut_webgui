use crate::{
  auth::user_session::UserSession,
  http::hypermedia::{error::ErrorPage, utils::RenderWithConfig},
  state::ServerState,
};
use askama::Template;
use axum::{
  Extension,
  extract::State,
  http::StatusCode,
  response::{Html, IntoResponse, Response},
};
use std::sync::Arc;

#[derive(Template)]
#[template(path = "not_found/+page.html")]
struct NotFound;

pub async fn get(
  state: State<Arc<ServerState>>,
  session: Option<Extension<UserSession>>,
) -> Result<Response, ErrorPage> {
  let response = (
    StatusCode::NOT_FOUND,
    Html(NotFound.render_with_config(&state.config, session.map(|v| v.0).as_ref())?),
  )
    .into_response();

  Ok(response)
}
