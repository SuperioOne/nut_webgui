use crate::{
  htmx_redirect,
  http_server::{
    ServerState, UpsdConfig,
    hypermedia::notifications::{Notification, NotificationTemplate},
  },
  ups_daemon_state::UpsEntry,
};
use askama::Template;
use axum::{
  Form,
  extract::{Path, Query, State},
  http::StatusCode,
  response::{Html, IntoResponse, Redirect, Response},
};
use nut_webgui_upsmc::{client::UpsAuthClient, errors::NutClientErrors};
use serde::Deserialize;
use tracing::{error, info};

#[derive(Deserialize)]
pub struct UpsFragmentQuery {
  section: Option<String>,
  tab: Option<String>,
}

#[derive(Deserialize)]
pub struct CommandRequest {
  command: String,
}

// If power is none, calculate power by using power_nominal and load values.
// if let Self {
//   power_nominal: Some(pw),
//   load: Some(ld),
//   power: None,
//   ..
// } = template
// {
//   template.power = Some((pw * ld) / 100.0_f64);
// };
//
// if let Self {
//   realpower_nominal: Some(pw),
//   load: Some(ld),
//   realpower: None,
//   ..
// } = template
// {
//   template.realpower = Some((pw * ld) / 100.0_f64);
// };
//

#[derive(Template)]
#[template(path = "ups/+page.html", ext = "html", escape = "none", blocks = ["ups_status", "tab_content"])]
struct UpsPageTemplate<'a> {
  tab_template: UpsPageTabTemplate<'a>,

  name: &'a str,
  desc: &'a str,
  model: Option<&'a str>,
  mfr: Option<&'a str>,
  ups_status: Option<String>,
  beeper_status: Option<bool>,

  poll_interval: u64,
}

#[derive(Template)]
enum UpsPageTabTemplate<'a> {
  #[template(source = "", ext = "html")]
  None,

  #[template(path = "ups/tab_commands.html")]
  Commands { ups: &'a UpsEntry },

  #[template(path = "ups/tab_variables.html")]
  Variables { ups: &'a UpsEntry },

  #[template(path = "ups/tab_grid.html")]
  Grid {
    battery_voltage: Option<f64>,
    charge: Option<f64>,
    charge_low: Option<f64>,
    input_voltage: Option<f64>,
    load: Option<f64>,
    power: Option<f64>,
    power_nominal: Option<f64>,
    realpower: Option<f64>,
    realpower_nominal: Option<f64>,
    runtime: Option<f64>,
    name: &'a str,
  },
}

#[inline]
fn get_tab_template<'a>(ups: &'a UpsEntry, tab_name: Option<&str>) -> UpsPageTabTemplate<'a> {
  match tab_name {
    Some("variables") => UpsPageTabTemplate::Variables { ups },
    Some("commands") => UpsPageTabTemplate::Commands { ups },
    _ => UpsPageTabTemplate::Grid {
      battery_voltage: None,
      charge: None,
      charge_low: None,
      input_voltage: None,
      load: None,
      power: None,
      power_nominal: None,
      realpower: None,
      realpower_nominal: None,
      runtime: None,
      name: &ups.name,
    },
  }
}

#[inline]
fn full_page_response(
  entry: Option<&UpsEntry>,
  tab_name: Option<&str>,
  upsd_config: &UpsdConfig,
) -> Response {
  if let Some(ups) = entry {
    let tab_template = get_tab_template(ups, tab_name);

    let template = UpsPageTemplate {
      beeper_status: Some(false),
      desc: &ups.desc,
      mfr: None,
      model: None,
      name: &ups.name,
      poll_interval: upsd_config.poll_interval.as_secs(),
      ups_status: Some("OL".to_owned()),

      tab_template,
    };

    Html(template.render().unwrap()).into_response()
  } else {
    Redirect::permanent("/not-found").into_response()
  }
}

#[inline]
fn partial_tab_content(entry: Option<&UpsEntry>, tab_name: Option<&str>) -> Response {
  if let Some(ups) = entry {
    let tab_template = get_tab_template(ups, tab_name);

    let template = UpsPageTemplate {
      beeper_status: None,
      desc: &ups.desc,
      mfr: None,
      model: None,
      name: &ups.name,
      poll_interval: 0,
      ups_status: None,

      tab_template,
    };

    Html(template.as_tab_content().render().unwrap()).into_response()
  } else {
    htmx_redirect!(StatusCode::NOT_FOUND, "/not-found").into_response()
  }
}

#[inline]
fn partial_ups_status(entry: Option<&UpsEntry>) -> Response {
  if let Some(ups) = entry {
    let template = UpsPageTemplate {
      name: &ups.name,
      desc: &ups.desc,
      ups_status: Some("OL".to_owned()),
      beeper_status: Some(false),

      tab_template: UpsPageTabTemplate::None,
      mfr: None,
      model: None,
      poll_interval: 0,
    };

    Html(template.as_ups_status().render().unwrap()).into_response()
  } else {
    htmx_redirect!(StatusCode::NOT_FOUND, "/not-found").into_response()
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
    Some("status") => partial_ups_status(ups_entry),
    Some("tabcontent") => partial_tab_content(ups_entry, query.tab.as_deref()),
    _ => full_page_response(ups_entry, query.tab.as_deref(), &state.upsd_config),
  }
}

pub async fn post_command(
  State(state): State<ServerState>,
  Path(ups_name): Path<String>,
  Form(request): Form<CommandRequest>,
) -> Response {
  let template: NotificationTemplate = {
    if let (Some(user), Some(pass)) = (&state.upsd_config.user, &state.upsd_config.pass) {
      match {
        let addr: &str = &state.upsd_config.addr;
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

  Html(template.render().unwrap()).into_response()
}
