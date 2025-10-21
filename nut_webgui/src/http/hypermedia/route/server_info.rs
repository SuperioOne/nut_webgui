use std::sync::Arc;

use crate::{
  auth::user_session::UserSession,
  config::{ServerConfig, UpsdConfig},
  http::hypermedia::{error::ErrorPage, utils::RenderWithConfig},
  state::{DaemonState, ServerState},
};
use askama::Template;
use axum::{
  Extension,
  extract::{Query, State},
  response::{Html, IntoResponse, Response},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ServerInfoFragmentQuery {
  section: Option<String>,
}

#[derive(Template)]
#[template(path = "server_info/+page.html", blocks = ["info_cards"])]
struct ServerInfoPageTemplate<'a> {
  server_config: &'a ServerConfig,
  upsd_html: Vec<String>,
}

#[derive(Template)]
#[template(path = "server_info/upsd_info.html")]
struct UpsdInfoTemplate<'a> {
  config: &'a UpsdConfig,
  state: &'a DaemonState,
}

pub async fn get(
  query: Query<ServerInfoFragmentQuery>,
  State(state): State<Arc<ServerState>>,
  session: Option<Extension<UserSession>>,
) -> Result<Response, ErrorPage> {
  let mut upsd_html = Vec::new();
  let session = session.map(|v| v.0);

  for upsd in state.upsd_servers.values() {
    let daemon_state = upsd.daemon_state.read().await;

    let upsd_info = UpsdInfoTemplate {
      state: &daemon_state,
      config: &upsd.config,
    }
    .render_with_config(&state.config, session.as_ref())?;

    upsd_html.push(upsd_info);
  }

  let template = ServerInfoPageTemplate {
    server_config: &state.config,
    upsd_html,
  };

  let response = match query.section.as_deref() {
    Some("info_cards") => Html(
      template
        .as_info_cards()
        .render_with_config(&state.config, session.as_ref())?,
    )
    .into_response(),
    _ => Html(template.render_with_config(&state.config, session.as_ref())?).into_response(),
  };

  Ok(response)
}
