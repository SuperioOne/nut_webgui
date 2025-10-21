use crate::{
  auth::user_session::UserSession,
  http::hypermedia::{
    error::ErrorPage,
    units::{
      ApparentPower, Approx, Celcius, OneOf, Percentage, RealPower, RemainingSeconds, UnitDisplay,
    },
    utils::{RenderWithConfig, normalize_id},
  },
  state::ServerState,
};
use askama::Template;
use axum::{
  Extension,
  extract::{Query, State},
  response::{Html, IntoResponse, Response},
};
use nut_webgui_upsmc::{UpsName, Value, VarName};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug)]
pub struct DeviceTableRow<'a> {
  id: String,
  status: Option<&'a str>,
  attached: usize,
  charge: Option<Percentage>,
  desc: &'a str,
  load: Option<Percentage>,
  name: &'a UpsName,
  power: Option<OneOf<ApparentPower, Approx<ApparentPower>>>,
  real_power: Option<OneOf<RealPower, Approx<RealPower>>>,
  runtime: Option<RemainingSeconds>,
  temperature: Option<Celcius>,
}

#[derive(Deserialize)]
pub struct HomeFragmentQuery {
  section: Option<String>,
}

struct RenderedRows<'a> {
  html: String,
  namespace: &'a str,
}

#[derive(Template)]
#[template(path = "+page.html", blocks = ["device_table"])]
struct HomeTemplate<'a> {
  rows_html: Vec<RenderedRows<'a>>,
}

#[derive(Template)]
#[template(path = "table_rows.html")]
struct TableRowsTemplate<'a> {
  devices: Vec<DeviceTableRow<'a>>,
  namespace: &'a str,
}

pub async fn get(
  query: Query<HomeFragmentQuery>,
  State(state): State<Arc<ServerState>>,
  session: Option<Extension<UserSession>>,
) -> Result<Response, ErrorPage> {
  let session = session.map(|v| v.0);
  let mut rendered_rows = Vec::with_capacity(state.upsd_servers.len());

  for upsd in state.upsd_servers.values() {
    let daemon_state = upsd.daemon_state.read().await;

    let mut devices: Vec<DeviceTableRow> = daemon_state
      .devices
      .values()
      .map(|device| {
        let status = {
          match device.variables.get(VarName::UPS_STATUS) {
            Some(Value::String(v)) => Some(v.as_ref()),
            _ => None,
          }
        };

        DeviceTableRow {
          id: format!(
            "{}{}",
            normalize_id(device.name.as_str()),
            normalize_id(&upsd.namespace)
          ),
          status,
          attached: device.attached.len(),
          charge: device.get_battery_charge(),
          desc: device.desc.as_ref(),
          load: device.get_ups_load(),
          name: &device.name,
          power: device.get_apparent_power(),
          real_power: device.get_real_power(),
          runtime: device.get_battery_runtime(),
          temperature: device.get_ups_temperature(),
        }
      })
      .collect();

    devices.sort_unstable_by_key(|v| v.name);

    let rows = TableRowsTemplate {
      devices,
      namespace: upsd.namespace.as_ref(),
    }
    .render_with_config(&state.config, session.as_ref())?;

    rendered_rows.push(RenderedRows {
      html: rows,
      namespace: upsd.namespace.as_ref(),
    });
  }

  rendered_rows.sort_unstable_by_key(|v| v.namespace);

  let template = HomeTemplate {
    rows_html: rendered_rows,
  };

  let response = match query.section.as_deref() {
    Some("device_table") => Html(
      template
        .as_device_table()
        .render_with_config(&state.config, session.as_ref())?,
    )
    .into_response(),
    _ => Html(template.render_with_config(&state.config, session.as_ref())?).into_response(),
  };

  Ok(response)
}
