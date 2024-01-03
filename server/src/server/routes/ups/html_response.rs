use std::sync::Arc;
use askama::Template;
use axum::extract::{Path, State};
use axum::http::{StatusCode};
use axum_core::response::IntoResponse;
use crate::htmx_redirect;
use crate::server::ups_info::{UpsInfo};
use crate::server::ServerState;
use crate::ups_mem_store::UpsEntry;

#[derive(Template)]
#[template(path = "ups/ups_info.html", ext = "html")]
struct UpsInfoTemplate {
  info: UpsInfo,
}

#[derive(Template)]
#[template(path = "ups/ups_variables.html", ext = "html")]
struct UpsVariableListTemplate {
  variables: Vec<(String, String)>,
}

#[derive(Template)]
#[template(path = "ups/+page.html", ext = "html", escape = "none")]
struct UpsPageTemplate<'a> {
  title: &'a str,
  info: UpsInfoTemplate,
  variables: UpsVariableListTemplate,
  commands: &'a [Box<str>],
}

impl<'a> UpsInfoTemplate {
  fn get_estimated_power(&self) -> Option<f64> {
    match (self.info.power_nominal, self.info.load) {
      (Some(power), Some(load)) => {
        let estimated = f64::from(power) * f64::from(load) / 100.0_f64;
        Some(estimated)
      }
      _ => None
    }
  }
}

fn create_var_template(ups: &UpsEntry) -> UpsVariableListTemplate {
  let variables: Vec<(String, String)> = ups.variables
    .iter()
    .map(|e| (e.name(), e.value_as_string()))
    .collect();

  UpsVariableListTemplate {
    variables,
  }
}

fn create_info_template(ups: &UpsEntry) -> UpsInfoTemplate {
  let ups_info = UpsInfo::load_from(ups);

  UpsInfoTemplate {
    info: ups_info
  }
}

pub(super) async fn page_response(Path(ups_name): Path<String>, State(state): State<Arc<ServerState>>) -> impl IntoResponse {
  let ups_name = ups_name.as_str();

  if let Some(ups) = state.store.read().await.get(ups_name) {
    let template = UpsPageTemplate {
      title: ups_name,
      info: create_info_template(ups),
      commands: &ups.commands,
      variables: create_var_template(ups),
    };

    template.into_response()
  } else {
    htmx_redirect!(StatusCode::NOT_FOUND, "/not-found").into_response()
  }
}

pub(super) async fn partial_page_variables(Path(ups_name): Path<String>, State(state): State<Arc<ServerState>>) -> impl IntoResponse {
  let ups_name = ups_name.as_str();

  if let Some(ups) = state.store.read().await.get(ups_name) {
    let ups_template = create_var_template(ups);

    ups_template.into_response()
  } else {
    htmx_redirect!(StatusCode::NOT_FOUND, "/not-found").into_response()
  }
}

pub(super) async fn partial_page_info(Path(ups_name): Path<String>, State(state): State<Arc<ServerState>>) -> impl IntoResponse {
  let ups_name = ups_name.as_str();

  if let Some(ups) = state.store.read().await.get(ups_name) {
    let ups_template = create_info_template(ups);

    ups_template.into_response()
  } else {
    htmx_redirect!(StatusCode::NOT_FOUND, "/not-found").into_response()
  }
}