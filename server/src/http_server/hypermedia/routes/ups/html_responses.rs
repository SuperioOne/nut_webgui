use crate::htmx_redirect;
use crate::http_server::hypermedia::ups_info::UpsInfo;
use crate::http_server::ServerState;
use crate::ups_mem_store::UpsEntry;
use askama::Template;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Redirect;
use axum_core::response::IntoResponse;
use std::borrow::Borrow;
use std::sync::Arc;

#[derive(Template)]
#[template(path = "ups/ups_info.html", ext = "html")]
struct UpsInfoTemplate {
  ups_info: UpsInfo,
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

impl UpsInfoTemplate {
  fn get_power(&self) -> Option<f64> {
    match self.ups_info {
      UpsInfo {
        power: Some(curr_power),
        ..
      } => Some(curr_power),
      UpsInfo {
        power_nominal: Some(pw),
        load: Some(ld),
        ..
      } => Some((pw * f64::from(ld)) / 100.0_f64),
      _ => None,
    }
  }
}

impl<T> From<T> for UpsVariableListTemplate
where
  T: Borrow<UpsEntry>,
{
  fn from(value: T) -> Self {
    let variables: Vec<(String, String)> = value
      .borrow()
      .variables
      .iter()
      .map(|e| (e.name(), e.value_as_string()))
      .collect();

    Self { variables }
  }
}

impl<T> From<T> for UpsInfoTemplate
where
  T: Borrow<UpsEntry>,
{
  fn from(value: T) -> Self {
    Self {
      ups_info: UpsInfo::from(value),
    }
  }
}

pub(super) async fn page_response(
  Path(ups_name): Path<String>,
  State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
  let ups_name = ups_name.as_str();

  if let Some(ups) = state.store.read().await.get(ups_name) {
    let template = UpsPageTemplate {
      title: ups_name,
      info: UpsInfoTemplate::from(ups),
      commands: &ups.commands,
      variables: UpsVariableListTemplate::from(ups),
    };

    template.into_response()
  } else {
    Redirect::permanent("/not-found").into_response()
  }
}

pub(super) async fn partial_ups_vars(
  Path(ups_name): Path<String>,
  State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
  let ups_name = ups_name.as_str();

  if let Some(ups) = state.store.read().await.get(ups_name) {
    UpsVariableListTemplate::from(ups).into_response()
  } else {
    htmx_redirect!(StatusCode::NOT_FOUND, "/not-found").into_response()
  }
}

pub(super) async fn partial_ups_info(
  Path(ups_name): Path<String>,
  State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
  let ups_name = ups_name.as_str();

  if let Some(ups) = state.store.read().await.get(ups_name) {
    UpsInfoTemplate::from(ups).into_response()
  } else {
    htmx_redirect!(StatusCode::NOT_FOUND, "/not-found").into_response()
  }
}
