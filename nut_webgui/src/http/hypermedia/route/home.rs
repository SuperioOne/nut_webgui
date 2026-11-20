use crate::{
  auth::user_session::UserSession,
  http::hypermedia::{
    error::ErrorPage,
    util::{named_enum::create_named_enum, render_extra::RenderWithConfig, unit::UnitDisplay},
  },
  state::{DeviceEntry, ServerState, UpsdState},
};
use askama::Template;
use axum::{
  Extension,
  extract::{Query, State},
  response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use std::{collections::BTreeMap, sync::Arc};
use upsmc::UpsName;

mod filters {
  use crate::http::hypermedia::util::normalize_id;
  use std::borrow::Cow;

  #[askama::filter_fn]
  pub fn id_filter<'a>(value: &'a str, _: &dyn askama::Values) -> askama::Result<Cow<'a, str>> {
    Ok(normalize_id(value))
  }
}

create_named_enum!(
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    named_enum HomePageSection {
        /// Main container with HMTX polling
        DeviceContainer = "device_container",
        /// The actual device list block with fuzzy search list
        DeviceList      = "device_list"
    }
);

#[derive(Deserialize)]
pub struct HomeFragmentQuery {
  section: Option<HomePageSection>,
  namespace: Option<Box<str>>,
}

#[derive(Template)]
#[template(path = "+page.html", blocks = ["device_list", "device_container"])]
struct HomeTemplate<'a> {
  rows_html: &'a [String],
  active_namespace: Option<&'a str>,
  state: &'a ServerState,
}

#[derive(Template)]
#[template(path = "table_rows.html")]
struct TableRowsTemplate<'a> {
  devices: &'a BTreeMap<UpsName, DeviceEntry>,
  namespace: &'a str,
}

async fn render_rows(
  upsd: &UpsdState,
  state: &ServerState,
  session: Option<&UserSession>,
) -> Result<Option<String>, askama::Error> {
  let daemon_state = upsd.daemon_state.read().await;

  if daemon_state.devices.is_empty() {
    Ok(None)
  } else {
    let rows = TableRowsTemplate {
      devices: &daemon_state.devices,
      namespace: upsd.namespace.as_ref(),
    }
    .render_with_config(&state.config, session)?;

    Ok(Some(rows))
  }
}

pub async fn get(
  query: Query<HomeFragmentQuery>,
  State(state): State<Arc<ServerState>>,
  session: Option<Extension<UserSession>>,
) -> Result<Response, ErrorPage> {
  let session = session.map(|v| v.0);
  let mut rendered_rows = Vec::new();

  match query
    .namespace
    .as_ref()
    .map(|n| state.upsd_servers.get(n.as_ref()))
    .flatten()
  {
    Some(upsd) => {
      if let Some(html) = render_rows(upsd.as_ref(), state.as_ref(), session.as_ref()).await? {
        rendered_rows.push(html);
      }
    }
    None => {
      rendered_rows.reserve(state.upsd_servers.len());

      for upsd in state.upsd_servers.values() {
        if let Some(html) = render_rows(upsd.as_ref(), state.as_ref(), session.as_ref()).await? {
          rendered_rows.push(html);
        }
      }
    }
  }

  let template = HomeTemplate {
    rows_html: &rendered_rows,
    active_namespace: query.namespace.as_deref(),
    state: &state,
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
