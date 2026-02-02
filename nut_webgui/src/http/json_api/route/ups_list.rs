use crate::{
  http::json_api::{problem_detail::ProblemDetail, route::extract_upsd},
  state::{DeviceEntry, ServerState},
};
use axum::{
  Json,
  extract::{Path, State},
  response::{IntoResponse, Response},
};
use std::sync::Arc;

pub async fn get(
  State(state): State<Arc<ServerState>>,
  Path(namespace): Path<Box<str>>,
) -> Result<Response, ProblemDetail> {
  let upsd = extract_upsd!(state, namespace.as_ref())?;
  let daemon_state = upsd.daemon_state.read().await;

  let mut device_refs: Vec<&DeviceEntry> = daemon_state.devices.values().collect();
  device_refs.sort_by(|r, l| r.name.cmp(&l.name));

  Ok(Json(device_refs).into_response())
}
