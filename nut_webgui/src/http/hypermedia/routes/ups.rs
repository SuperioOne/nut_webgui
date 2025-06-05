use std::collections::HashMap;

use crate::{
  config::{ServerConfig, UpsdConfig},
  device_entry::DeviceEntry,
  htmx_redirect,
  http::{
    RouterState,
    hypermedia::notifications::{Notification, NotificationTemplate},
  },
  state::{DescriptionKey, ServerState},
};
use askama::Template;
use axum::{
  Form,
  extract::{Path, Query, State},
  http::StatusCode,
  response::{Html, IntoResponse, Redirect, Response},
};
use nut_webgui_upsmc::{CmdName, UpsName, Value, VarName, clients::NutAuthClient};
use serde::Deserialize;
use tracing::{error, info};

#[derive(Deserialize)]
pub struct UpsFragmentQuery {
  section: Option<String>,
  tab: Option<String>,
}

#[derive(Deserialize)]
pub struct CommandRequest {
  command: CmdName,
}

#[derive(Template)]
#[template(path = "ups/+page.html", ext = "html", blocks = ["ups_status", "tab_content"])]
struct UpsPageTemplate<'a> {
  tab_template: UpsPageTabTemplate<'a>,

  name: &'a UpsName,
  desc: &'a str,
  model: Option<&'a Value>,
  mfr: Option<&'a Value>,
  ups_status: Option<String>,
  beeper_status: Option<bool>,

  poll_interval: u64,
  base_path: &'a str,
  default_theme: Option<&'a str>,
}

#[derive(Template)]
enum UpsPageTabTemplate<'a> {
  #[template(source = "", ext = "html")]
  None,

  #[template(path = "ups/tab_commands.html")]
  Commands {
    device: &'a DeviceEntry,
    descriptions: &'a HashMap<DescriptionKey, Box<str>>,
  },

  #[template(path = "ups/tab_variables.html")]
  Variables {
    variables: Vec<(&'a VarName, &'a Value)>,
    descriptions: &'a HashMap<DescriptionKey, Box<str>>,
  },

  #[template(path = "ups/tab_grid.html")]
  Grid { device: &'a DeviceEntry },
}

#[inline]
fn get_tab_template<'a>(
  device: &'a DeviceEntry,
  tab_name: Option<&str>,
  state: &'a ServerState,
) -> UpsPageTabTemplate<'a> {
  match tab_name {
    Some("variables") => {
      let mut variables: Vec<_> = device.variables.iter().collect();
      variables.sort_unstable_by_key(|(k, _)| *k);

      UpsPageTabTemplate::Variables {
        variables,
        descriptions: &state.shared_desc,
      }
    }
    Some("commands") => UpsPageTabTemplate::Commands {
      device,
      descriptions: &state.shared_desc,
    },
    _ => UpsPageTabTemplate::Grid { device },
  }
}

#[inline]
fn full_page_response(
  entry: Option<&DeviceEntry>,
  tab_name: Option<&str>,
  state: &ServerState,
  config: &ServerConfig,
) -> Response {
  if let Some(ups) = entry {
    let tab_template = get_tab_template(ups, tab_name, state);

    let template = UpsPageTemplate {
      beeper_status: Some(false),
      desc: &ups.desc,
      mfr: ups.variables.get(VarName::DEVICE_MFR),
      model: ups.variables.get(VarName::DEVICE_MODEL),
      name: &ups.name,
      poll_interval: config.upsd.poll_interval,
      ups_status: Some("OL".to_owned()),

      tab_template,
      default_theme: config.default_theme.as_deref(),
      base_path: config.http_server.base_path.as_str(),
    };

    Html(template.render().unwrap()).into_response()
  } else {
    Redirect::permanent("/not-found").into_response()
  }
}

#[inline]
fn partial_tab_content(
  entry: Option<&DeviceEntry>,
  tab_name: Option<&str>,
  state: &ServerState,
  config: &ServerConfig,
) -> Response {
  if let Some(ups) = entry {
    let tab_template = get_tab_template(ups, tab_name, state);

    let template = UpsPageTemplate {
      beeper_status: None,
      desc: &ups.desc,
      mfr: ups.variables.get(VarName::DEVICE_MFR),
      model: ups.variables.get(VarName::DEVICE_MODEL),
      name: &ups.name,
      poll_interval: 0,
      ups_status: None,

      tab_template,
      default_theme: config.default_theme.as_deref(),
      base_path: config.http_server.base_path.as_str(),
    };

    Html(template.as_tab_content().render().unwrap()).into_response()
  } else {
    htmx_redirect!(StatusCode::NOT_FOUND, "/not-found").into_response()
  }
}

#[inline]
fn partial_ups_status(entry: Option<&DeviceEntry>) -> Response {
  if let Some(ups) = entry {
    let template = UpsPageTemplate {
      name: &ups.name,
      desc: &ups.desc,
      ups_status: Some("OL".to_owned()),
      beeper_status: Some(false),

      tab_template: UpsPageTabTemplate::None,
      mfr: ups.variables.get(VarName::DEVICE_MFR),
      model: ups.variables.get(VarName::DEVICE_MODEL),
      poll_interval: 0,

      default_theme: None,
      base_path: "",
    };

    Html(template.as_ups_status().render().unwrap()).into_response()
  } else {
    htmx_redirect!(StatusCode::NOT_FOUND, "/not-found").into_response()
  }
}

pub async fn get(
  Path(ups_name): Path<UpsName>,
  query: Query<UpsFragmentQuery>,
  rs: State<RouterState>,
) -> Response {
  let state = rs.state.read().await;
  let ups_entry = state.devices.get(&ups_name);

  match query.section.as_deref() {
    Some("status") => partial_ups_status(ups_entry),
    Some("tabcontent") => partial_tab_content(ups_entry, query.tab.as_deref(), &state, &rs.config),
    _ => full_page_response(ups_entry, query.tab.as_deref(), &state, &rs.config),
  }
}

pub async fn post_command(
  State(rs): State<RouterState>,
  Path(ups_name): Path<UpsName>,
  Form(request): Form<CommandRequest>,
) -> Response {
  let (addr, username, password) = match &rs.config.upsd {
    upsd @ UpsdConfig {
      pass: Some(pass),
      user: Some(user),
      ..
    } => (upsd.get_socket_addr(), user.as_ref(), pass.as_ref()),
    _ => {
      return Html(
        NotificationTemplate::new(
          "No username or password configured for UPS daemon. Server is in read-only mode.".into(),
          Notification::Info,
          None,
        )
        .render()
        .unwrap(),
      )
      .into_response();
    }
  };

  let connection = NutAuthClient::connect(addr, username, password).await;

  let cmd_result = match connection {
    Ok(mut client) => {
      let result = client.instcmd(&ups_name, &request.command).await;
      _ = client.close().await;
      result
    }
    Err(err) => Err(err),
  };

  let template = match cmd_result {
    Ok(_) => {
      info!(message = "instcmd called successfully", device_name = %ups_name, cmd = %request.command);

      NotificationTemplate::new(
        format!(
          "Command '{0}' successfully executed for UPS '{1}'.",
          &request.command, &ups_name
        ),
        Notification::Success,
        None,
      )
    }
    Err(err) => {
      error!(message = "instcmd call failed", device_name = %ups_name, cmd = %request.command, reason = %err);

      NotificationTemplate::new(
        format!("INSTCMD call failed, {:?}", err),
        Notification::Error,
        None,
      )
    }
  };

  Html(template.render().unwrap()).into_response()
}
