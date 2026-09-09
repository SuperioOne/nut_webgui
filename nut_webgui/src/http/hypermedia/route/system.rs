use crate::{
  auth::{user_session::UserSession, user_store::UserStore},
  config::ServerConfig,
  http::hypermedia::{error::ErrorPage, util::RenderWithConfig},
  state::ServerState,
};
use askama::Template;
use axum::{
  Extension,
  extract::State,
  response::{Html, IntoResponse, Response},
};
use std::sync::Arc;

#[derive(Template)]
#[template(path = "system/+page.html")]
struct SystemPageTemplate<'a> {
  config: &'a ServerConfig,
  users: Option<&'a UserStore>,
}

pub async fn get(
  State(state): State<Arc<ServerState>>,
  session: Option<Extension<UserSession>>,
) -> Result<Response, ErrorPage> {
  let session = session.map(|v| v.0);

  let template = SystemPageTemplate {
    config: &state.config,
    users: state.auth_user_store.as_ref().map(|v| v.as_ref()),
  };

  let response =
    Html(template.render_with_config(&state.config, session.as_ref())?).into_response();

  Ok(response)
}
