use std::sync::Arc;

use crate::{
  auth::user_session::UserSession,
  config::UpsdConfig,
  http::hypermedia::{error::ErrorPage, util::RenderWithConfig},
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
pub struct ConnectionFragmentQuery {
  section: Option<String>,
}

struct RenderedUpsdInfo<'a> {
  html: String,
  name: &'a str,
}

#[derive(Template)]
#[template(path = "connection/+page.html", blocks = ["info_cards"])]
struct ConnectionPageTemplate<'a> {
  upsd_html: Vec<RenderedUpsdInfo<'a>>,
}

#[derive(Template)]
#[template(path = "connection/upsd_info.html")]
struct UpsdInfoTemplate<'a> {
  config: &'a UpsdConfig,
  state: &'a DaemonState,
  name: &'a str,
}

pub async fn get(
  query: Query<ConnectionFragmentQuery>,
  State(state): State<Arc<ServerState>>,
  session: Option<Extension<UserSession>>,
) -> Result<Response, ErrorPage> {
  let mut upsd_html = Vec::new();
  let session = session.map(|v| v.0);

  for upsd in state.upsd_servers.values() {
    let daemon_state = upsd.daemon_state.read().await;

    let html = UpsdInfoTemplate {
      state: &daemon_state,
      config: &upsd.config,
      name: &upsd.namespace,
    }
    .render_with_config(&state.config, session.as_ref())?;

    upsd_html.push(RenderedUpsdInfo {
      html,
      name: &upsd.namespace,
    });
  }

  upsd_html.sort_unstable_by_key(|v| v.name);

  let template = ConnectionPageTemplate { upsd_html };

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
