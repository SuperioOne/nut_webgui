use crate::state::DaemonStatus;

use super::RouterState;
use axum::{
  Json,
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse<'a> {
  last_device_sync: Option<&'a DateTime<Utc>>,
  upsd_port: u16,
  upsd_server: &'a str,
  upsd_status: DaemonStatus,
}

pub async fn get_health(State(state): State<RouterState>) -> Response {
  let upsd_state = state.state.read().await;

  let response = Json(HealthResponse {
    last_device_sync: upsd_state.remote_state.last_device_sync.as_ref(),
    upsd_server: &state.config.upsd.addr,
    upsd_port: state.config.upsd.port,
    upsd_status: upsd_state.remote_state.status,
  });

  if response.upsd_status != DaemonStatus::Dead {
    (StatusCode::OK, response).into_response()
  } else {
    (StatusCode::INTERNAL_SERVER_ERROR, response).into_response()
  }
}

pub async fn get_readiness(State(state): State<RouterState>) -> Response {
  let upsd_state = state.state.read().await;

  if upsd_state.remote_state.status == DaemonStatus::Online {
    (StatusCode::OK, "READY").into_response()
  } else {
    (StatusCode::SERVICE_UNAVAILABLE, "NOT READY").into_response()
  }
}
