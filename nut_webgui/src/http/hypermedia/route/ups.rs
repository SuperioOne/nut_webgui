use crate::{
  auth::user_session::UserSession,
  config::UpsdConfig,
  device_entry::{DeviceEntry, VarDetail},
  http::hypermedia::{
    error::ErrorPage,
    notification::NotificationTemplate,
    semantic_type::SemanticType,
    unit::UnitDisplay,
    util::{RenderWithConfig, redirect_not_found},
  },
  state::{DescriptionKey, ServerState},
};
use askama::Template;
use axum::{
  Extension,
  extract::{Path, Query, State},
  response::{Html, IntoResponse, Response},
};
use nut_webgui_upsmc::{UpsName, Value, VarName};
use serde::{Deserialize, de::Visitor};
use std::{
  collections::{BTreeMap, HashMap},
  sync::Arc,
};
use tokio::sync::RwLockReadGuard;

pub mod fsd;
pub mod instcmd;
pub mod rw;

#[derive(Template, Debug)]
#[template(path = "ups/+page.html", ext = "html", blocks = ["ups_status", "tab_content"])]
struct UpsPageTemplate<'a> {
  device: &'a DeviceEntry,
  namespace: &'a str,
  tab_template: UpsPageTabTemplate<'a>,
  upsd_config: &'a UpsdConfig,
}

#[derive(Template, Debug)]
#[template(path = "ups/form_rw.html")]
pub struct RwFormTemplate<'a> {
  pub detail: &'a VarDetail,
  pub message: Option<&'static str>,
  pub semantic: SemanticType,
  pub value: Option<&'a Value>,
  pub device_name: &'a UpsName,
  pub namespace: &'a str,
  pub var_name: &'a VarName,
  pub notification: Option<NotificationTemplate<'a>>,
}

#[derive(Template, Debug)]
enum UpsPageTabTemplate<'a> {
  #[template(source = "", ext = "html")]
  None,

  #[template(path = "ups/tab_commands.html")]
  Commands {
    descriptions: RwLockReadGuard<'a, HashMap<DescriptionKey, Box<str>>>,
    device: &'a DeviceEntry,
    namespace: &'a str,
  },

  #[template(path = "ups/tab_variables.html")]
  Variables {
    descriptions: RwLockReadGuard<'a, HashMap<DescriptionKey, Box<str>>>,
    name: &'a UpsName,
    namespace: &'a str,
    variables: Vec<(&'a VarName, &'a Value)>,
    upsd_config: &'a UpsdConfig,
  },

  #[template(path = "ups/tab_grid.html")]
  Grid {
    device: &'a DeviceEntry,
    namespace: &'a str,
    upsd_config: &'a UpsdConfig,
  },

  #[template(path = "ups/tab_rw.html")]
  Rw {
    descriptions: RwLockReadGuard<'a, HashMap<DescriptionKey, Box<str>>>,
    inputs: BTreeMap<VarName, RwFormTemplate<'a>>,
    name: &'a UpsName,
  },

  #[template(path = "ups/tab_clients.html")]
  Clients {
    device: &'a DeviceEntry,
    namespace: &'a str,
    upsd_config: &'a UpsdConfig,
  },
}

impl UpsPageTabTemplate<'_> {
  fn as_type_str(&self) -> &'static str {
    match self {
      UpsPageTabTemplate::None => "none",
      UpsPageTabTemplate::Commands { .. } => "commands",
      UpsPageTabTemplate::Variables { .. } => "variables",
      UpsPageTabTemplate::Grid { .. } => "grid",
      UpsPageTabTemplate::Rw { .. } => "rw",
      UpsPageTabTemplate::Clients { .. } => "clients",
    }
  }
}

async fn get_tab_template<'a>(
  tab_name: TabName,
  namespace: &'a str,
  device: &'a DeviceEntry,
  server_state: &'a ServerState,
  upsd_config: &'a UpsdConfig,
) -> UpsPageTabTemplate<'a> {
  match tab_name {
    TabName::Variables => {
      let mut variables: Vec<_> = device.variables.iter().collect();
      variables.sort_unstable_by_key(|(k, _)| *k);

      let descriptions = server_state.shared_desc.read().await;

      UpsPageTabTemplate::Variables {
        namespace,
        variables,
        name: &device.name,
        descriptions,
        upsd_config,
      }
    }
    TabName::Commands => {
      let descriptions = server_state.shared_desc.read().await;

      UpsPageTabTemplate::Commands {
        descriptions,
        device,
        namespace,
      }
    }
    TabName::Clients => UpsPageTabTemplate::Clients {
      device,
      namespace,
      upsd_config,
    },
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
            notification: None,
            semantic: SemanticType::None,
            value,
            namespace,
            var_name: name,
          };

          (name.clone(), input)
        })
        .collect();

      let descriptions = server_state.shared_desc.read().await;

      UpsPageTabTemplate::Rw {
        descriptions,
        inputs,
        name: &device.name,
      }
    }
    _ => UpsPageTabTemplate::Grid {
      device,
      namespace,
      upsd_config,
    },
  }
}

#[derive(Deserialize)]
pub struct UpsFragmentQuery {
  section: Option<Box<str>>,
  tab: Option<TabName>,
}

pub async fn get(
  Path((namespace, ups_name)): Path<(Box<str>, UpsName)>,
  query: Query<UpsFragmentQuery>,
  state: State<Arc<ServerState>>,
  session: Option<Extension<UserSession>>,
) -> Result<Response, ErrorPage> {
  let upsd = match state.upsd_servers.get(&namespace) {
    Some(upsd) => upsd,
    None => return Ok(redirect_not_found!(&state)),
  };

  let daemon_state = upsd.daemon_state.read().await;
  let device = match daemon_state.devices.get(&ups_name) {
    Some(ups) => ups,
    None => return Ok(redirect_not_found!(&state)),
  };

  let tab_name = query.tab.unwrap_or(TabName::Grid);
  let session = session.map(|v| v.0);

  let mut template = UpsPageTemplate {
    device,
    tab_template: UpsPageTabTemplate::None,
    upsd_config: &upsd.config,
    namespace: &namespace,
  };

  let response = match query.section.as_deref() {
    Some("status") => Html(
      template
        .as_ups_status()
        .render_with_config(&state.config, session.as_ref())?,
    )
    .into_response(),
    Some("tab_content") => {
      template.tab_template =
        get_tab_template(tab_name, &namespace, device, &state, &upsd.config).await;

      Html(
        template
          .as_tab_content()
          .render_with_config(&state.config, session.as_ref())?,
      )
      .into_response()
    }
    _ => {
      template.tab_template =
        get_tab_template(tab_name, &namespace, device, &state, &upsd.config).await;

      Html(template.render_with_config(&state.config, session.as_ref())?).into_response()
    }
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
