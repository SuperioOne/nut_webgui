use crate::{http_server::ServerState, ups_daemon_state::UpsEntry};
use askama::Template;
use axum::{
  extract::{Query, State},
  response::{Html, IntoResponse, Response},
};
use nut_webgui_upsmc::ups_variables::UpsVariable;
use serde::Deserialize;

#[derive(Debug)]
pub struct UpsTableRow<'a> {
  charge: Option<f64>,
  desc: &'a str,
  load: Option<f64>,
  name: &'a str,
  status: Option<&'a str>,
}

impl<'a> UpsTableRow<'a> {
  pub fn from_ups_entry(ups: &UpsEntry) -> UpsTableRow {
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
          row.status = Some(val.as_str());
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

#[derive(Deserialize)]
pub struct HomeFragmentQuery {
  section: Option<String>,
}

#[derive(Template)]
#[template(path = "+page.html", blocks = ["ups_table"])]
struct HomeTemplate<'a> {
  ups_list: Vec<UpsTableRow<'a>>,
}

pub async fn get(query: Query<HomeFragmentQuery>, State(state): State<ServerState>) -> Response {
  let upsd_state = &state.upsd_state.read().await;
  let mut ups_list: Vec<UpsTableRow> = upsd_state
    .iter()
    .map(|(_, ups)| UpsTableRow::from(ups))
    .collect();

  ups_list.sort_unstable_by_key(|v| v.name);

  let template = HomeTemplate { ups_list };

  match query.section.as_deref() {
    Some("ups_table") => Html(template.as_ups_table().render().unwrap()).into_response(),
    _ => Html(template.render().unwrap()).into_response(),
  }
}
