use crate::{config::ServerConfig, http::RouterState, state::DaemonState};
use askama::Template;
use axum::{
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
struct ServerInfoTemplate<'a> {
  base_path: &'a str,
  default_theme: Option<&'a str>,

  device_count: usize,
  config: &'a ServerConfig,
  state: &'a DaemonState,
}

pub async fn get(query: Query<ServerInfoFragmentQuery>, State(rs): State<RouterState>) -> Response {
  let state = &rs.state.read().await;

  let template = ServerInfoTemplate {
    base_path: rs.config.http_server.base_path.as_str(),
    default_theme: rs.config.default_theme.as_deref(),

    config: &rs.config,
    state: &state.remote_state,
    device_count: state.devices.len(),
  };

  match query.section.as_deref() {
    Some("info_cards") => Html(template.as_info_cards().render().unwrap()).into_response(),
    _ => Html(template.render().unwrap()).into_response(),
  }
}
