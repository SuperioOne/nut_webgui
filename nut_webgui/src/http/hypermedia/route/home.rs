use crate::{
  auth::user_session::UserSession,
  http::hypermedia::{
    error::ErrorPage,
    unit::UnitDisplay,
    util::{RenderWithConfig, normalize_id},
  },
  state::{DeviceEntry, ServerState, UpsdState},
};
use askama::Template;
use axum::{
  Extension,
  extract::{Query, State},
  response::{Html, IntoResponse, Response},
};
use nut_webgui_upsmc::Value;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug)]
pub struct DeviceTableRow<'a> {
  id: String,
  device: &'a DeviceEntry,
}

#[derive(Deserialize)]
pub struct HomeFragmentQuery {
  section: Option<Box<str>>,
  namespace: Option<Box<str>>,
}

struct RenderedRows<'a> {
  html: String,
  namespace: &'a str,
}

#[derive(Template)]
#[template(path = "+page.html", blocks = ["device_table", "device_container"])]
struct HomeTemplate<'a> {
  rows_html: Vec<RenderedRows<'a>>,
  active_namespace: Option<&'a str>,
  namespaces: Vec<&'a str>,
}

#[derive(Template)]
#[template(path = "table_rows.html")]
struct TableRowsTemplate<'a> {
  devices: Vec<DeviceTableRow<'a>>,
  namespace: &'a str,
}

async fn render_device_rows(
  upsd: &UpsdState,
  state: &ServerState,
  session: Option<&UserSession>,
) -> Result<String, askama::Error> {
  let daemon_state = upsd.daemon_state.read().await;

  let mut devices: Vec<DeviceTableRow> = daemon_state
    .devices
    .values()
    .map(|device| DeviceTableRow {
      id: format!(
        "{}@{}",
        normalize_id(device.name.as_str()),
        normalize_id(&upsd.namespace)
      ),
      device,
    })
    .collect();

  devices.sort_unstable_by_key(|v| v.device.name.as_str());

  TableRowsTemplate {
    devices,
    namespace: upsd.namespace.as_ref(),
  }
  .render_with_config(&state.config, session)
}

pub async fn get(
  query: Query<HomeFragmentQuery>,
  State(state): State<Arc<ServerState>>,
  session: Option<Extension<UserSession>>,
) -> Result<Response, ErrorPage> {
  let session = session.map(|v| v.0);
  let mut rendered_rows = Vec::with_capacity(state.upsd_servers.len());

  match query
    .namespace
    .as_ref()
    .map(|n| state.upsd_servers.get(n.as_ref()))
    .flatten()
  {
    Some(upsd) => {
      let html = render_device_rows(upsd.as_ref(), state.as_ref(), session.as_ref()).await?;

      rendered_rows.push(RenderedRows {
        html,
        namespace: upsd.namespace.as_ref(),
      });
    }
    None => {
      for upsd in state.upsd_servers.values() {
        let html = render_device_rows(upsd.as_ref(), state.as_ref(), session.as_ref()).await?;

        rendered_rows.push(RenderedRows {
          html,
          namespace: upsd.namespace.as_ref(),
        });
      }
    }
  }

  rendered_rows.sort_unstable_by_key(|v| v.namespace);

  let mut namespaces: Vec<&str> = state.upsd_servers.keys().map(|v| v.as_ref()).collect();
  namespaces.sort_unstable();

  let template = HomeTemplate {
    rows_html: rendered_rows,
    active_namespace: query.namespace.as_deref(),
    namespaces,
  };

  let response = match query.section.as_deref() {
    Some("device_table") => Html(
      template
        .as_device_table()
        .render_with_config(&state.config, session.as_ref())?,
    )
    .into_response(),
    Some("device_container") => Html(
      template
        .as_device_container()
        .render_with_config(&state.config, session.as_ref())?,
    )
    .into_response(),

    _ => Html(template.render_with_config(&state.config, session.as_ref())?).into_response(),
  };

  Ok(response)
}
