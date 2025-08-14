use crate::{
  config::{ServerConfig, UpsdConfig},
  device_entry::{DeviceEntry, VarDetail},
  htmx_redirect, htmx_swap,
  http::{
    RouterState,
    hypermedia::{
      error::ErrorPage, notifications::NotificationTemplate, semantic_classes::SemanticType,
      utils::RenderWithConfig,
    },
  },
  state::{DescriptionKey, ServerState},
};
use askama::Template;
use axum::{
  Form,
  extract::{Path, Query, State},
  http::{HeaderValue, StatusCode},
  response::{Html, IntoResponse, Redirect, Response},
};
use nut_webgui_upsmc::{CmdName, InferValueFrom, UpsName, Value, VarName, clients::NutAuthClient};
use serde::{Deserialize, de::Visitor};
use std::{
  collections::{BTreeMap, HashMap},
  time::Duration,
};
use tracing::{error, info};

#[derive(Template, Debug)]
#[template(path = "ups/+page.html", ext = "html", blocks = ["ups_status", "tab_content"])]
struct UpsPageTemplate<'a> {
  device: &'a DeviceEntry,
  tab_template: UpsPageTabTemplate<'a>,
}

#[derive(Template, Debug)]
#[template(path = "ups/form_rw.html")]
pub struct RwFormTemplate<'a> {
  pub detail: &'a VarDetail,
  pub message: Option<&'static str>,
  pub semantic: SemanticType,
  pub value: Option<&'a Value>,
  pub device_name: &'a UpsName,
  pub var_name: &'a VarName,
  pub notification: Option<NotificationTemplate<'a>>,
}

#[derive(Debug)]
struct CmdTemplate<'a> {
  id: &'a str,
  desc: Option<&'a str>,
}

#[derive(Template, Debug)]
enum UpsPageTabTemplate<'a> {
  #[template(source = "", ext = "html")]
  None,

  #[template(path = "ups/tab_commands.html")]
  Commands {
    device: &'a DeviceEntry,
    commands: Vec<CmdTemplate<'a>>,
  },

  #[template(path = "ups/tab_variables.html")]
  Variables {
    variables: Vec<(&'a VarName, &'a Value)>,
    descriptions: &'a HashMap<DescriptionKey, Box<str>>,
    name: &'a UpsName,
  },

  #[template(path = "ups/tab_grid.html")]
  Grid { device: &'a DeviceEntry },

  #[template(path = "ups/tab_rw.html")]
  Rw {
    name: &'a UpsName,
    inputs: BTreeMap<VarName, RwFormTemplate<'a>>,
    descriptions: &'a HashMap<DescriptionKey, Box<str>>,
  },

  #[template(path = "ups/tab_clients.html")]
  Clients { device: &'a DeviceEntry },
}

#[inline]
fn get_tab_template<'a>(
  device: &'a DeviceEntry,
  tab_name: TabName,
  state: &'a ServerState,
) -> UpsPageTabTemplate<'a> {
  match tab_name {
    TabName::Variables => {
      let mut variables: Vec<_> = device.variables.iter().collect();
      variables.sort_unstable_by_key(|(k, _)| *k);

      UpsPageTabTemplate::Variables {
        variables,
        name: &device.name,
        descriptions: &state.shared_desc,
      }
    }
    TabName::Commands => {
      let cmds = device
        .commands
        .iter()
        .map(|c| {
          let desc = state.shared_desc.get(c.as_str()).map(|v| v.as_ref());
          CmdTemplate { id: c.as_str(), desc }
        })
        .collect();
      UpsPageTabTemplate::Commands {
        device,
        commands: cmds,
      }
    }
    TabName::Clients => UpsPageTabTemplate::Clients { device },
    TabName::Rw => {
      let inputs = device
        .rw_variables
        .iter()
        .map(|(name, detail)| {
          let value = device.variables.get(name);
          let input = RwFormTemplate {
            detail,
            device_name: &device.name,
            message: None,
            semantic: SemanticType::None,
            value,
            var_name: name,
            notification: None,
          };

          (name.clone(), input)
        })
        .collect();

      UpsPageTabTemplate::Rw {
        inputs,
        name: &device.name,
        descriptions: &state.shared_desc,
      }
    }
    _ => UpsPageTabTemplate::Grid { device },
  }
}

#[inline]
fn full_page_response(
  entry: Option<&DeviceEntry>,
  tab_name: TabName,
  state: &ServerState,
  config: &ServerConfig,
) -> Result<Response, ErrorPage<askama::Error>> {
  let response = if let Some(device) = entry {
    let tab_template = get_tab_template(device, tab_name, state);

    let template = UpsPageTemplate {
      device,
      tab_template,
    };

    Html(template.render_with_config(config)?).into_response()
  } else {
    Redirect::permanent(&format!("{}/not-found", config.http_server.base_path)).into_response()
  };

  Ok(response)
}

#[inline]
fn partial_tab_content(
  entry: Option<&DeviceEntry>,
  tab_name: TabName,
  state: &ServerState,
  config: &ServerConfig,
) -> Result<Response, ErrorPage<askama::Error>> {
  let response = if let Some(device) = entry {
    let tab_template = get_tab_template(device, tab_name, state);

    let template = UpsPageTemplate {
      device,
      tab_template,
    };

    Html(template.as_tab_content().render_with_config(config)?).into_response()
  } else {
    htmx_redirect!(
      StatusCode::NOT_FOUND,
      format!("{}/not-found", config.http_server.base_path)
    )
    .into_response()
  };

  Ok(response)
}

#[inline]
fn partial_ups_status(
  entry: Option<&DeviceEntry>,
  config: &ServerConfig,
) -> Result<Response, ErrorPage<askama::Error>> {
  let response = if let Some(device) = entry {
    let template = UpsPageTemplate {
      device,
      tab_template: UpsPageTabTemplate::None,
    };

    Html(template.as_ups_status().render_with_config(config)?).into_response()
  } else {
    htmx_redirect!(
      StatusCode::NOT_FOUND,
      format!("{}/not-found", config.http_server.base_path)
    )
    .into_response()
  };

  Ok(response)
}

#[derive(Deserialize)]
pub struct UpsFragmentQuery {
  section: Option<Box<str>>,
  tab: Option<TabName>,
}

pub async fn get(
  Path(ups_name): Path<UpsName>,
  query: Query<UpsFragmentQuery>,
  rs: State<RouterState>,
) -> Result<Response, ErrorPage<askama::Error>> {
  let tab_name = query.tab.unwrap_or(TabName::Grid);
  let state = rs.state.read().await;
  let ups_entry = state.devices.get(&ups_name);
  match query.section.as_deref() {
    Some("status") => partial_ups_status(ups_entry, &rs.config),
    Some("tab_content") => partial_tab_content(ups_entry, tab_name, &state, &rs.config),
    _ => full_page_response(ups_entry, tab_name, &state, &rs.config),
  }
}

#[derive(Deserialize, Debug)]
pub struct CommandRequest {
  command: CmdName,
}

pub async fn post_instcmd(
  State(rs): State<RouterState>,
  Path(ups_name): Path<UpsName>,
  Form(request): Form<CommandRequest>,
) -> Result<Response, ErrorPage<askama::Error>> {
  let (addr, username, password) = match &rs.config.upsd {
    upsd @ UpsdConfig {
      pass: Some(pass),
      user: Some(user),
      ..
    } => (upsd.get_socket_addr(), user.as_ref(), pass.as_ref()),
    _ => {
      return Ok(
        Html(
          NotificationTemplate::from(
            "No username or password configured for UPS daemon. Server is in read-only mode.",
          )
          .render_with_config(&rs.config)?,
        )
        .into_response(),
      );
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

      NotificationTemplate::from(format!(
        "'{0}' successfully executed on {1}.",
        &request.command, &ups_name
      ))
      .set_level(SemanticType::Success)
    }
    Err(err) => {
      error!(message = "instcmd call failed", device_name = %ups_name, cmd = %request.command, reason = %err);

      NotificationTemplate::from(format!("INSTCMD call failed, {}", err))
        .set_level(SemanticType::Error)
    }
  };

  Ok(Html(template.render_with_config(&rs.config)?).into_response())
}

pub async fn post_fsd(
  State(rs): State<RouterState>,
  Path(ups_name): Path<UpsName>,
) -> Result<Response, ErrorPage<askama::Error>> {
  let (addr, username, password) = match &rs.config.upsd {
    upsd @ UpsdConfig {
      pass: Some(pass),
      user: Some(user),
      ..
    } => (upsd.get_socket_addr(), user.as_ref(), pass.as_ref()),
    _ => {
      return Ok(
        Html(
          NotificationTemplate::from(
            "No username or password configured for UPS daemon. Server is in read-only mode.",
          )
          .render_with_config(&rs.config)?,
        )
        .into_response(),
      );
    }
  };

  let connection = NutAuthClient::connect(addr, username, password).await;

  let fsd_result = match connection {
    Ok(mut client) => {
      let result = client.fsd(&ups_name).await;
      _ = client.close().await;
      result
    }
    Err(err) => Err(err),
  };

  let template = match fsd_result {
    Ok(_) => {
      info!(message = "forced-shutdown called successfully", device_name = %ups_name);

      NotificationTemplate::from(format!("FSD flag set on {0}.", &ups_name))
        .set_level(SemanticType::Warning)
    }
    Err(err) => {
      error!(message = "fsd call failed", device_name = %ups_name, reason = %err);

      NotificationTemplate::from(format!("FSD failed, {}", err)).set_level(SemanticType::Error)
    }
  };

  Ok(Html(template.render_with_config(&rs.config)?).into_response())
}

#[derive(Deserialize, Debug)]
pub struct RwRequest {
  name: VarName,
  value: Box<str>,
}

pub async fn patch_rw(
  State(rs): State<RouterState>,
  Path(ups_name): Path<UpsName>,
  Form(request): Form<RwRequest>,
) -> Result<Response, ErrorPage<askama::Error>> {
  let (addr, username, password) = match &rs.config.upsd {
    upsd @ UpsdConfig {
      pass: Some(pass),
      user: Some(user),
      ..
    } => (upsd.get_socket_addr(), user.as_ref(), pass.as_ref()),
    _ => {
      return Ok(htmx_swap!(
        Html(
          NotificationTemplate::from(
            "No username or password configured for UPS daemon. Server is in read-only mode.",
          )
          .render_with_config(&rs.config)?
        ),
        "none"
      ));
    }
  };

  let state = rs.state.read().await;
  let detail = match state.devices.get(&ups_name) {
    Some(device) => match device.rw_variables.get(&request.name) {
      Some(detail) => detail,
      None => {
        return Ok(htmx_swap!(
          Html(
            NotificationTemplate::from(format!(
              "device driver does not support write on {}",
              request.name
            ))
            .set_level(SemanticType::Error)
            .render_with_config(&rs.config)?
          ),
          "none"
        ));
      }
    },
    None => {
      return Ok(
        htmx_redirect!(
          StatusCode::NOT_FOUND,
          format!("{}/not-found", rs.config.http_server.base_path)
        )
        .into_response(),
      );
    }
  };

  let (value, message, semantic, is_valid) = match detail {
    VarDetail::String { max_len } => {
      if request.value.as_ref().trim().is_empty() {
        (
          Value::from(request.value),
          Some("input is empty"),
          SemanticType::Error,
          false,
        )
      } else if request.value.as_ref().len() > *max_len {
        (
          Value::from(request.value),
          Some("input is too long"),
          SemanticType::Error,
          false,
        )
      } else {
        (Value::from(request.value), None, SemanticType::None, true)
      }
    }
    VarDetail::Number => match Value::infer_number_from(request.value.as_ref()) {
      Ok(v) => (v, None, SemanticType::None, true),
      Err(_) => (
        Value::from(request.value),
        Some("input is not a number"),
        SemanticType::Error,
        false,
      ),
    },
    VarDetail::Enum { options } => {
      let value = Value::from(request.value);

      if options.contains(&value) {
        (value, None, SemanticType::None, true)
      } else {
        (value, Some("invalid option"), SemanticType::Error, false)
      }
    }
    VarDetail::Range { min, max } => match Value::infer_number_from(request.value.as_ref()) {
      Ok(value) => match (min.as_lossly_f64(), max.as_lossly_f64()) {
        (Some(min), Some(max)) => {
          let valuef64 = value.as_lossly_f64().unwrap_or(0.0);

          if min <= valuef64 && valuef64 <= max {
            (value, None, SemanticType::None, true)
          } else {
            (
              value,
              Some("value is not in range"),
              SemanticType::Error,
              false,
            )
          }
        }
        _ => (
          value,
          Some("driver reported min-max values are not numeric values"),
          SemanticType::Error,
          false,
        ),
      },
      Err(_) => (
        Value::from(request.value),
        Some("input is not a number"),
        SemanticType::Error,
        false,
      ),
    },
  };

  let response = if is_valid {
    let connection = NutAuthClient::connect(addr, username, password).await;

    match connection {
      Ok(mut client) => {
        let result = client.set_var(&ups_name, &request.name, &value).await;
        _ = client.close().await;

        let (semantic, message, notification) = match result {
          Ok(_) => {
            info!(message = "set var request accepted", device = %ups_name, value = %value, name = %request.name);

            (
              semantic,
              message,
              Some(
                NotificationTemplate::from("Set var request is accepted")
                  .set_level(SemanticType::Success),
              ),
            )
          }
          Err(err) => {
            error!(message = "set var request failed", device = %ups_name,  value = %value, name = %request.name, reason = %err);

            (
              SemanticType::Error,
              Some("value is rejected by device driver"),
              Some(
                NotificationTemplate::from(format!("Set var request failed, {}", err))
                  .set_level(SemanticType::Error)
                  .set_ttl(Duration::from_secs(15)),
              ),
            )
          }
        };

        Html(
          RwFormTemplate {
            value: Some(&value),
            semantic,
            message,
            detail,
            device_name: &ups_name,
            var_name: &request.name,
            notification,
          }
          .render_with_config(&rs.config)?,
        )
        .into_response()
      }
      Err(err) => {
        error!(message = "auth connection failed for set var request", value = %value, name = %request.name, reason = %err);

        htmx_swap!(
          Html(
            NotificationTemplate::from(format!("client authentication failed, {}", err))
              .set_level(SemanticType::Error)
              .render_with_config(&rs.config)?,
          ),
          "none"
        )
      }
    }
  } else {
    Html(
      RwFormTemplate {
        value: Some(&value),
        semantic,
        message,
        detail,
        device_name: &ups_name,
        var_name: &request.name,
        notification: Some(
          NotificationTemplate::from("Input validation failed").set_level(SemanticType::Error),
        ),
      }
      .render_with_config(&rs.config)?,
    )
    .into_response()
  };

  Ok(response)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TabName {
  Commands,
  Clients,
  Grid,
  Rw,
  Unknown,
  Variables,
}

struct TabNameVisitor;

impl<'de> Deserialize<'de> for TabName {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_str(TabNameVisitor)
  }
}

impl<'de> Visitor<'de> for TabNameVisitor {
  type Value = TabName;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("expecting tab name string")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    match v.to_ascii_lowercase().as_str() {
      "commands" => Ok(TabName::Commands),
      "clients" => Ok(TabName::Clients),
      "grid" => Ok(TabName::Grid),
      "rw" => Ok(TabName::Rw),
      "variables" => Ok(TabName::Variables),
      _ => Ok(TabName::Unknown),
    }
  }

  #[inline]
  fn visit_borrowed_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    self.visit_str(v)
  }

  #[inline]
  fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    self.visit_str(v.as_str())
  }
}
