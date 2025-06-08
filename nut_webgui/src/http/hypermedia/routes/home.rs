use crate::{
  device_entry::DeviceEntry,
  http::{
    RouterState,
    hypermedia::{device_entry_impls::ValueDetail, filters::normalize_id},
  },
};
use askama::Template;
use axum::{
  extract::{Query, State},
  response::{Html, IntoResponse, Response},
};
use nut_webgui_upsmc::{UpsName, Value, VarName};
use serde::Deserialize;
use std::borrow::Cow;

#[derive(Debug)]
pub struct DeviceTableRow<'a> {
  id: Cow<'a, str>,
  attached: usize,
  charge: Option<ValueDetail<'a>>,
  desc: &'a str,
  load: Option<ValueDetail<'a>>,
  name: &'a UpsName,
  runtime: Option<ValueDetail<'a>>,
  status: Option<&'a str>,
  temperature: Option<ValueDetail<'a>>,
  power: Option<ValueDetail<'a>>,
}

impl<'a> From<&'a DeviceEntry> for DeviceTableRow<'a> {
  #[inline]
  fn from(device: &'a DeviceEntry) -> Self {
    let charge = device.get_battery_charge();
    let load = device.get_ups_load();
    let runtime = device.get_battery_runtime();
    let temperature = device.get_ups_temperature();
    let power = device.get_power();

    let status = {
      match device.variables.get(VarName::UPS_STATUS) {
        Some(Value::String(v)) => Some(v.as_ref()),
        _ => None,
      }
    };

    DeviceTableRow {
      id: normalize_id(device.name.as_str()),
      attached: device.attached.len(),
      charge,
      desc: device.desc.as_ref(),
      load,
      name: &device.name,
      runtime,
      status,
      temperature,
      power,
    }
  }
}

#[derive(Deserialize)]
pub struct HomeFragmentQuery {
  section: Option<String>,
}

#[derive(Template)]
#[template(path = "+page.html", blocks = ["device_table"])]
struct HomeTemplate<'a> {
  base_path: &'a str,
  default_theme: Option<&'a str>,
  devices: Vec<DeviceTableRow<'a>>,
}

pub async fn get(query: Query<HomeFragmentQuery>, State(rs): State<RouterState>) -> Response {
  let state = &rs.state.read().await;
  let mut device_list: Vec<DeviceTableRow> = state
    .devices
    .iter()
    .map(|(_, device)| DeviceTableRow::from(device))
    .collect();

  device_list.sort_unstable_by_key(|v| v.name);

  let template = HomeTemplate {
    devices: device_list,
    base_path: rs.config.http_server.base_path.as_str(),
    default_theme: rs.config.default_theme.as_deref(),
  };

  match query.section.as_deref() {
    Some("device_table") => Html(template.as_device_table().render().unwrap()).into_response(),
    _ => Html(template.render().unwrap()).into_response(),
  }
}
