use crate::{
  auth::user_session::UserSession,
  http::hypermedia::{error::ErrorPage, not_found::NotFound},
  state::ServerState,
};
use axum::{
  Extension,
  extract::State,
  response::{IntoResponse as _, Response},
};
use std::sync::Arc;

pub async fn get(
  state: State<Arc<ServerState>>,
  session: Option<Extension<UserSession>>,
) -> Result<Response, ErrorPage> {
  Ok(NotFound::new_response(&state.config, session.as_ref().map(|v| &v.0))?.into_response())
}
