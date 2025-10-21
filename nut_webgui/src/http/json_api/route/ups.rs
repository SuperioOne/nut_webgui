use crate::{
  http::json_api::{problem_detail::ProblemDetail, route::extract_upsd},
  state::ServerState,
};
use axum::{
  Json,
  extract::{Path, State, rejection::PathRejection},
  http::StatusCode,
  response::{IntoResponse, Response},
};
use nut_webgui_upsmc::UpsName;
use std::sync::Arc;

pub async fn get(
  State(state): State<Arc<ServerState>>,
  paths: Result<Path<(Box<str>, UpsName)>, PathRejection>,
) -> Result<Response, ProblemDetail> {
  let Path((namespace, ups_name)) = paths?;
  let upsd = extract_upsd!(state, namespace)?;

  match upsd.daemon_state.read().await.devices.get(&ups_name) {
    Some(ups) => Ok(Json(ups).into_response()),
    None => Err(ProblemDetail::new(
      "Device not found",
      StatusCode::NOT_FOUND,
    )),
  }
}
