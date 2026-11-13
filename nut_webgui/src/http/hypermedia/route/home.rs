use crate::{
  auth::user_session::UserSession,
  http::hypermedia::{
    error::ErrorPage,
    util::{
      named_enum::create_named_enum, normalize_id, render_extra::RenderWithConfig,
      unit::UnitDisplay,
    },
  },
  state::{ConnectionStatus, DeviceEntry, ServerState, StaleDevice, UpsdState},
};
use askama::Template;
use axum::{
  Extension,
  extract::{Query, State},
  response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use std::sync::Arc;

create_named_enum!(
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    named_enum HomePageSection {
        /// Main container with HMTX polling
        DeviceContainer = "device_container",
        /// The actual device list block with fuzzy search list
        DeviceList      = "device_list"
    }
);

#[derive(Debug)]
struct DeviceTableRow<'a> {
  id: String,
  device: &'a DeviceEntry,
}

/// A device that was known before the connection to its daemon was lost.
#[derive(Debug)]
struct StaleTableRow<'a> {
  id: String,
  device: &'a StaleDevice,
}

#[derive(Deserialize)]
pub struct HomeFragmentQuery {
  section: Option<HomePageSection>,
  namespace: Option<Box<str>>,
}

struct RenderedRows<'a> {
  html: String,
  namespace: &'a str,
}

#[derive(Template)]
#[template(path = "+page.html", blocks = ["device_list", "device_container"])]
struct HomeTemplate<'a> {
  rows_html: Vec<RenderedRows<'a>>,
  active_namespace: Option<&'a str>,
  namespaces: Vec<&'a str>,
}

#[derive(Template)]
#[template(path = "table_rows.html")]
struct TableRowsTemplate<'a> {
  devices: Vec<DeviceTableRow<'a>>,
  stale_devices: Vec<StaleTableRow<'a>>,
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

  // NOTE: Devices are removed when the connection to the daemon is lost. Instead of making them
  // disappear, the last known devices are listed as grayed-out rows.
  let stale_devices: Vec<StaleTableRow> = if daemon_state.status != ConnectionStatus::Online {
    daemon_state
      .stale_devices
      .iter()
      .filter(|device| !daemon_state.devices.contains_key(&device.name))
      .map(|device| StaleTableRow {
        id: format!(
          "{}@{}",
          normalize_id(device.name.as_str()),
          normalize_id(&upsd.namespace)
        ),
        device,
      })
      .collect()
  } else {
    Vec::new()
  };

  TableRowsTemplate {
    devices,
    stale_devices,
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

  let body = match query.section {
    Some(HomePageSection::DeviceList) => template
      .as_device_list()
      .render_with_config(&state.config, session.as_ref()),
    Some(HomePageSection::DeviceContainer) => template
      .as_device_container()
      .render_with_config(&state.config, session.as_ref()),
    _ => template.render_with_config(&state.config, session.as_ref()),
  }?;

  Ok(Html(body).into_response())
}
