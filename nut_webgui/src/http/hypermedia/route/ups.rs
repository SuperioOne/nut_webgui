use crate::{
  auth::user_session::UserSession,
  config::UpsdConfig,
  http::hypermedia::{
    error::ErrorPage,
    not_found::NotFound,
    notification::NotificationTemplate,
    util::{
      named_enum::create_named_enum, render_extra::RenderWithConfig, semantic_type::SemanticType,
      unit::UnitDisplay,
    },
  },
  state::{DescriptionKey, DeviceEntry, ServerState, VarDetail},
};
use askama::Template;
use axum::{
  Extension,
  extract::{Path, Query, State},
  response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use std::{
  collections::{BTreeMap, HashMap},
  sync::Arc,
};
use tokio::sync::RwLockReadGuard;
use upsmc::{UpsName, Value, VarName};

pub mod fsd;
pub mod instcmd;
pub mod rw;

create_named_enum!(
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    named_enum UpsPageSection {
        /// Tab body
        TabContent = "tab_content",
        /// UPS  tatus indicator
        Status     = "status"
    }
);

create_named_enum!(
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    named_enum TabName {
        Commands  = "commands",
        Clients   = "clients",
        Grid      = "grid",
        Rw        = "rw",
        Variables = "variables"
    }
);

#[derive(Template, Debug)]
#[template(path = "ups/+page.html", ext = "html", blocks = ["ups_status", "tab_content"])]
struct UpsPageTemplate<'a> {
  device: &'a DeviceEntry,
  namespace: &'a str,
  tab_template: Option<UpsPageTabTemplate<'a>>,
  upsd_config: &'a UpsdConfig,
}

#[derive(Template, Debug)]
#[template(path = "ups/form_rw.html")]
struct RwFormTemplate<'a> {
  detail: &'a VarDetail,
  device_name: &'a UpsName,
  message: Option<&'static str>,
  namespace: &'a str,
  notification: Option<NotificationTemplate<'a>>,
  semantic: SemanticType,
  value: Option<&'a Value>,
  var_name: &'a VarName,
}

#[derive(Template, Debug)]
enum UpsPageTabTemplate<'a> {
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

#[derive(Deserialize, Clone, Copy)]
pub struct UpsFragmentQuery {
  section: Option<UpsPageSection>,
  tab: Option<TabName>,
}

impl UpsPageTabTemplate<'_> {
  const fn get_tab_name(&self) -> TabName {
    match self {
      UpsPageTabTemplate::Commands { .. } => TabName::Commands,
      UpsPageTabTemplate::Variables { .. } => TabName::Variables,
      UpsPageTabTemplate::Grid { .. } => TabName::Grid,
      UpsPageTabTemplate::Rw { .. } => TabName::Rw,
      UpsPageTabTemplate::Clients { .. } => TabName::Clients,
    }
  }
}

pub async fn get(
  Path((namespace, ups_name)): Path<(Box<str>, UpsName)>,
  query: Query<UpsFragmentQuery>,
  state: State<Arc<ServerState>>,
  session: Option<Extension<UserSession>>,
) -> Result<Response, ErrorPage> {
  let tab_name = query.tab.unwrap_or(TabName::Grid);
  let session = session.map(|v| v.0);
  let upsd = match state.upsd_servers.get(namespace.as_ref()) {
    Some(upsd) => upsd,
    None => return Ok(NotFound::new_response(&state.config, session.as_ref())?.into_response()),
  };
  let daemon_state = upsd.daemon_state.read().await;
  let device = match daemon_state.devices.get(&ups_name) {
    Some(ups) => ups,
    None => return Ok(NotFound::new_response(&state.config, session.as_ref())?.into_response()),
  };

  let mut template = UpsPageTemplate {
    device,
    tab_template: None,
    upsd_config: &upsd.config,
    namespace: &namespace,
  };

  if query.section.is_none_or(|v| v != UpsPageSection::Status) {
    let tab_template = match tab_name {
      TabName::Variables => {
        let mut variables: Vec<_> = device.variables.iter().collect();
        variables.sort_unstable_by_key(|(k, _)| *k);

        UpsPageTabTemplate::Variables {
          namespace: &namespace,
          variables,
          name: &device.name,
          descriptions: state.shared_desc.read().await,
          upsd_config: &upsd.config,
        }
      }
      TabName::Commands => UpsPageTabTemplate::Commands {
        descriptions: state.shared_desc.read().await,
        device: device,
        namespace: &namespace,
      },
      TabName::Clients => UpsPageTabTemplate::Clients {
        device: device,
        namespace: &namespace,
        upsd_config: &upsd.config,
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
              namespace: &namespace,
              var_name: name,
            };

            (name.clone(), input)
          })
          .collect();

        UpsPageTabTemplate::Rw {
          descriptions: state.shared_desc.read().await,
          inputs,
          name: &device.name,
        }
      }
      _ => UpsPageTabTemplate::Grid {
        device: device,
        namespace: &namespace,
        upsd_config: &upsd.config,
      },
    };

    template.tab_template = Some(tab_template)
  }

  let body = match query.section {
    Some(UpsPageSection::Status) => template
      .as_ups_status()
      .render_with_config(&state.config, session.as_ref()),
    Some(UpsPageSection::TabContent) => template
      .as_tab_content()
      .render_with_config(&state.config, session.as_ref()),
    _ => template.render_with_config(&state.config, session.as_ref()),
  }?;

  Ok(Html(body).into_response())
}
