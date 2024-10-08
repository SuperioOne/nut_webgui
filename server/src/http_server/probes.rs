use super::ServerState;
use crate::ups_daemon_state::DaemonStatus;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse<'a> {
  upsd_status: DaemonStatus,
  last_modified: Option<&'a DateTime<Utc>>,
  upsd_server: &'a str,
}

pub async fn get_health(State(state): State<ServerState>) -> impl IntoResponse {
  let upsd_state = state.upsd_state.read().await;

  let response = Json(HealthResponse {
    last_modified: upsd_state.last_modified.as_ref(),
    upsd_server: &state.upsd_config.addr,
    upsd_status: upsd_state.status,
  });

  if response.upsd_status != DaemonStatus::Dead {
    (StatusCode::OK, response).into_response()
  } else {
    (StatusCode::INTERNAL_SERVER_ERROR, response).into_response()
  }
}

pub async fn get_readiness(State(state): State<ServerState>) -> impl IntoResponse {
  let upsd_state = state.upsd_state.read().await;

  if upsd_state.status == DaemonStatus::Online {
    StatusCode::OK
  } else {
    StatusCode::INTERNAL_SERVER_ERROR
  }
}
