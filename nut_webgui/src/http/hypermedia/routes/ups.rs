use crate::{
  config::ServerConfig,
  device_entry::{DeviceEntry, VarDetail},
  htmx_redirect,
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
  extract::{Path, Query, State},
  http::StatusCode,
  response::{Html, IntoResponse, Redirect, Response},
};
use nut_webgui_upsmc::{UpsName, Value, VarName};
use serde::{Deserialize, de::Visitor};
use std::collections::{BTreeMap, HashMap};

pub mod fsd;
pub mod instcmd;
pub mod rw;

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

#[derive(Template, Debug)]
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
    TabName::Commands => UpsPageTabTemplate::Commands {
      device,
      descriptions: &state.shared_desc,
    },
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

fn full_page_response(
  entry: Option<&DeviceEntry>,
  tab_name: TabName,
  state: &ServerState,
  config: &ServerConfig,
) -> Result<Response, ErrorPage> {
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

fn partial_tab_content(
  entry: Option<&DeviceEntry>,
  tab_name: TabName,
  state: &ServerState,
  config: &ServerConfig,
) -> Result<Response, ErrorPage> {
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

fn partial_ups_status(
  entry: Option<&DeviceEntry>,
  config: &ServerConfig,
) -> Result<Response, ErrorPage> {
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
) -> Result<Response, ErrorPage> {
  let state = rs.state.read().await;
  let ups_entry = state.devices.get(&ups_name);
  let tab_name = query.tab.unwrap_or(TabName::Grid);

  match query.section.as_deref() {
    Some("status") => partial_ups_status(ups_entry, &rs.config),
    Some("tab_content") => partial_tab_content(ups_entry, tab_name, &state, &rs.config),
    _ => full_page_response(ups_entry, tab_name, &state, &rs.config),
  }
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
