use crate::{
  config::ServerConfig,
  http::{
    RouterState,
    hypermedia::{error::ErrorPage, utils::RenderWithConfig},
  },
  state::DaemonState,
};
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
  device_count: usize,
  config: &'a ServerConfig,
  state: &'a DaemonState,
}

pub async fn get(
  query: Query<ServerInfoFragmentQuery>,
  State(rs): State<RouterState>,
) -> Result<Response, ErrorPage<askama::Error>> {
  let state = &rs.state.read().await;

  let template = ServerInfoTemplate {
    config: &rs.config,
    state: &state.remote_state,
    device_count: state.devices.len(),
  };

  let response = match query.section.as_deref() {
    Some("info_cards") => {
      Html(template.as_info_cards().render_with_config(&rs.config)?).into_response()
    }
    _ => Html(template.render_with_config(&rs.config)?).into_response(),
  };

  Ok(response)
}
