use crate::{
  http_server::ServerState, ups_daemon_state::UpsEntry, upsd_client::ups_variables::UpsVariable,
};
use askama::Template;
use axum::{
  extract::{Query, State},
  http::HeaderMap,
  response::Response,
};
use axum_core::response::IntoResponse;
use serde::Deserialize;

#[derive(Debug)]
pub struct UpsTableRow<'a> {
  charge: Option<f64>,
  desc: &'a str,
  load: Option<f64>,
  name: &'a str,
  status: Option<String>,
}

impl<'a> UpsTableRow<'a> {
  pub fn from_ups_entry(ups: &'a UpsEntry) -> UpsTableRow {
    let mut row = UpsTableRow {
      charge: None,
      desc: &ups.desc,
      load: None,
      name: &ups.name,
      status: None,
    };

    for variable in &ups.variables {
      match variable {
        UpsVariable::UpsLoad(val) => {
          row.load = Some(*val);
        }
        UpsVariable::BatteryCharge(val) => {
          row.charge = Some(*val);
        }
        UpsVariable::UpsStatus(val) => {
          row.status = Some(val.to_string());
        }
        _ => {}
      }
    }

    row
  }
}

impl<'a> From<&'a UpsEntry> for UpsTableRow<'a> {
  fn from(value: &'a UpsEntry) -> Self {
    Self::from_ups_entry(value)
  }
}

#[derive(Template)]
#[template(path = "ups_table.html", ext = "html")]
struct UpsTableTemplate<'a> {
  ups_list: Vec<UpsTableRow<'a>>,
  url_prefix: &'a str,
}

#[derive(Deserialize)]
pub struct HomeFragmentQuery {
  section: Option<String>,
}

#[derive(Template)]
#[template(path = "+page.html", escape = "none", ext = "html")]
struct HomeTemplate<'a> {
  title: &'a str,
  ups_table: UpsTableTemplate<'a>,
  url_prefix: &'a str,
}

pub async fn get(
  query: Query<HomeFragmentQuery>,
  State(state): State<ServerState>,
  headers: HeaderMap,
) -> Response {
  let script_name = headers
    .get("x-script-name")
    .and_then(|v| v.to_str().ok())
    .unwrap_or("");
  let upsd_state = &state.upsd_state.read().await;
  let mut ups_list: Vec<UpsTableRow> = upsd_state
    .iter()
    .map(|(_, ups)| UpsTableRow::from(ups))
    .collect();

  ups_list.sort_unstable_by_key(|v| v.name);

  let table_template = UpsTableTemplate {
    ups_list,
    url_prefix: script_name,
  };

  match query.section.as_deref() {
    Some("ups_table") => table_template.into_response(),
    _ => HomeTemplate {
      title: "Home",
      ups_table: table_template,
      url_prefix: script_name,
    }
    .into_response(),
  }
}
