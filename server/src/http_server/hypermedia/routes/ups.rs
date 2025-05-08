use crate::{
  htmx_redirect,
  http_server::{
    hypermedia::notifications::{Notification, NotificationTemplate},
    ServerConfig, ServerState,
  },
  ups_daemon_state::UpsEntry,
  upsd_client::{client::UpsAuthClient, errors::NutClientErrors, ups_variables::UpsVariable},
};
use askama::Template;
use askama_axum::Response;
use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  response::Redirect,
  Form,
};
use axum_core::response::IntoResponse;
use serde::Deserialize;
use tracing::{error, info};

#[derive(Deserialize)]
pub struct UpsFragmentQuery {
  section: Option<String>,
}

#[derive(Deserialize)]
pub struct CommandRequest {
  command: String,
}

#[derive(Template)]
#[template(path = "ups/ups_status.html", ext = "html")]
struct UpsStatusTemplate {
  ups_status: Option<String>,
  beeper_status: Option<bool>,
}

impl Default for UpsStatusTemplate {
  fn default() -> Self {
    Self {
      beeper_status: None,
      ups_status: None,
    }
  }
}

impl From<&UpsEntry> for UpsStatusTemplate {
  fn from(entry: &UpsEntry) -> Self {
    let mut template = Self::default();

    for variable in entry.variables.iter() {
      match variable {
        UpsVariable::UpsBeeperStatus(beeper_status) => {
          template.beeper_status = match beeper_status.as_str() {
            "enabled" => Some(true),
            _ => Some(false),
          };
        }
        UpsVariable::UpsStatus(ups_status) => {
          template.ups_status = Some(ups_status.to_string());
        }
        _ => {}
      }
    }

    template
  }
}

#[derive(Template)]
#[template(path = "ups/ups_info.html", ext = "html", escape = "none")]
struct UpsInfoTemplate<'a> {
  title: &'a str,
  base_path: &'a str,
  battery_voltage: Option<f64>,
  charge: Option<f64>,
  charge_low: Option<f64>,
  desc: &'a str,
  input_voltage: Option<f64>,
  load: Option<f64>,
  name: &'a str,
  model: Option<&'a str>,
  mfr: Option<&'a str>,
  realpower: Option<f64>,
  realpower_nominal: Option<f64>,
  power: Option<f64>,
  power_nominal: Option<f64>,
  runtime: Option<f64>,
  variables: Vec<(&'a str, String)>,
  ups_status_template: UpsStatusTemplate,
  hx_status_interval: u64,
}

impl<'a> UpsInfoTemplate<'a> {
  pub fn from_ups_entry(ups: &'a UpsEntry, configs: &'a ServerConfig) -> Self {
    let variables: Vec<(&'a str, String)> = ups
      .variables
      .iter()
      .map(|e| (e.name(), e.value_as_string()))
      .collect();

    let mut template = Self {
      title: &ups.name,
      base_path: &configs.base_path,
      battery_voltage: None,
      charge: None,
      charge_low: None,
      desc: &ups.desc,
      input_voltage: None,
      load: None,
      name: &ups.name,
      model: None,
      mfr: None,
      realpower: None,
      realpower_nominal: None,
      power: None,
      power_nominal: None,
      runtime: None,
      variables,
      ups_status_template: UpsStatusTemplate::default(),
      hx_status_interval: configs.poll_interval.as_secs(),
    };

    for variable in ups.variables.iter() {
      match variable {
        UpsVariable::UpsLoad(val) => {
          template.load = Some(*val);
        }
        UpsVariable::UpsRealPowerNominal(val) => {
          template.realpower_nominal = Some(*val);
        }
        UpsVariable::UpsRealPower(val) => {
          template.realpower = Some(*val);
        }
        UpsVariable::UpsPower(val) => {
          template.power = Some(*val);
        }
        UpsVariable::UpsPowerNominal(val) => {
          template.power_nominal = Some(*val);
        }
        UpsVariable::BatteryCharge(val) => {
          template.charge = Some(*val);
        }
        UpsVariable::BatteryChargeLow(val) => {
          template.charge_low = Some(*val);
        }
        UpsVariable::BatteryRuntime(val) => {
          template.runtime = Some(*val);
        }
        UpsVariable::UpsStatus(val) => {
          template.ups_status_template.ups_status = Some(val.to_string());
        }
        UpsVariable::BatteryVoltage(val) => {
          template.battery_voltage = Some(*val);
        }
        UpsVariable::InputVoltage(val) => {
          template.input_voltage = Some(*val);
        }
        UpsVariable::UpsMfr(val) => {
          template.mfr = Some(val);
        }
        UpsVariable::UpsModel(val) => {
          template.model = Some(val);
        }
        UpsVariable::UpsBeeperStatus(val) => {
          template.ups_status_template.beeper_status = match val.as_str() {
            "enabled" => Some(true),
            _ => Some(false),
          };
        }
        _ => {}
      }
    }

    // If power is none, calculate power by using power_nominal and load values.
    if let Self {
      power_nominal: Some(pw),
      load: Some(ld),
      power: None,
      ..
    } = template
    {
      template.power = Some((pw * ld) / 100.0_f64);
    };

    if let Self {
      realpower_nominal: Some(pw),
      load: Some(ld),
      realpower: None,
      ..
    } = template
    {
      template.realpower = Some((pw * ld) / 100.0_f64);
    };

    template
  }
}

// TODO: Switch to block fragments when askama v0.13 released
#[derive(Template)]
#[template(path = "ups/+page.html", ext = "html", escape = "none")]
struct UpsPageTemplate<'a> {
  title: &'a str,
  base_path: &'a str,
  ups_info: UpsInfoTemplate<'a>,
  commands: &'a [Box<str>],
  hx_info_interval: u64,
}

#[inline]
fn page_response(entry: Option<&UpsEntry>, configs: &ServerConfig) -> Response {
  if let Some(ups) = entry {
    UpsPageTemplate {
      base_path: &configs.base_path,
      commands: &ups.commands,
      hx_info_interval: configs.poll_freq.as_secs(),
      title: &ups.name,
      ups_info: UpsInfoTemplate::from_ups_entry(ups, configs),
    }
    .into_response()
  } else {
    Redirect::permanent(&format!("{}/not-found", configs.base_path)).into_response()
  }
}

#[inline]
fn partial_ups_info(entry: Option<&UpsEntry>, configs: &ServerConfig) -> Response {
  if let Some(ups) = entry {
    UpsInfoTemplate::from_ups_entry(ups, configs).into_response()
  } else {
    htmx_redirect!(
      StatusCode::NOT_FOUND,
      format!("{}/not-found", configs.base_path)
    )
    .into_response()
  }
}

#[inline]
fn partial_ups_status(entry: Option<&UpsEntry>, configs: &ServerConfig) -> Response {
  if let Some(ups) = entry {
    UpsStatusTemplate::from(ups).into_response()
  } else {
    htmx_redirect!(
      StatusCode::NOT_FOUND,
      format!("{}/not-found", configs.base_path)
    )
    .into_response()
  }
}

pub async fn get(
  Path(ups_name): Path<String>,
  query: Query<UpsFragmentQuery>,
  state: State<ServerState>,
) -> Response {
  let upsd_state = state.upsd_state.read().await;
  let ups_entry = upsd_state.get_ups(&ups_name);

  match query.section.as_deref() {
    Some("info") => partial_ups_info(ups_entry, &state.configs),
    Some("status") => partial_ups_status(ups_entry, &state.configs),
    _ => page_response(ups_entry, &state.configs),
  }
}

pub async fn post_command(
  State(state): State<ServerState>,
  Path(ups_name): Path<String>,
  Form(request): Form<CommandRequest>,
) -> Response {
  let template: NotificationTemplate = {
    if let (Some(user), Some(pass)) = (&state.configs.user, &state.configs.pass) {
      match {
        let addr: &str = &state.configs.addr;
        let ups_name: &str = &ups_name;
        let cmd: &str = &request.command;

        async move {
          let mut client = UpsAuthClient::create(addr, user, pass).await?;
          client.send_instcmd(ups_name, cmd).await?;
          info!("INSTCMD '{0}' called for UPS '{1}'", cmd, ups_name);
          Ok::<(), NutClientErrors>(())
        }
      }
      .await
      {
        Ok(_) => NotificationTemplate::new(
          format!(
            "Command '{0}' successfully executed for UPS '{1}'.",
            &request.command, &ups_name
          ),
          Notification::Success,
          None,
        ),
        Err(err) => {
          error!(message = "INSTCMD call failed.", ups_name = ups_name, reason = %err);

          NotificationTemplate::new(
            format!("INSTCMD call failed, {:?}", err),
            Notification::Error,
            None,
          )
        }
      }
    } else {
      NotificationTemplate::new(
        "No username or password configured for UPS daemon. Server is in read-only mode.".into(),
        Notification::Info,
        None,
      )
    }
  };

  template.into_response()
}
