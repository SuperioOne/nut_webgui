use std::sync::Arc;

use crate::state::{ConnectionStatus, ServerState};
use axum::{
  Json,
  extract::{Path, State},
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse<T, S>
where
  T: Serialize,
  S: Serialize,
{
  last_device_sync: T,
  upsd_port: u16,
  upsd_server: S,
  upsd_status: ConnectionStatus,
}

pub async fn get_health(State(state): State<Arc<ServerState>>) -> Response {
  let mut active_count: usize = 0;
  let mut health_status = Vec::with_capacity(state.upsd_servers.len());

  for upsd in state.upsd_servers.values() {
    let daemon_state = upsd.daemon_state.read().await;

    let upsd_health = HealthResponse {
      last_device_sync: daemon_state.last_device_sync.clone(),
      upsd_port: upsd.config.port,
      upsd_server: upsd.config.addr.as_ref(),
      upsd_status: daemon_state.status,
    };

    health_status.push(upsd_health);

    if daemon_state.status != ConnectionStatus::Dead {
      active_count += 1;
    }
  }

  if active_count > 0 {
    (StatusCode::OK, Json(health_status)).into_response()
  } else {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(health_status)).into_response()
  }
}

pub async fn get_readiness(State(state): State<Arc<ServerState>>) -> Response {
  let mut ready_count: usize = 0;

  for upsd in state.upsd_servers.values() {
    let daemon_state = upsd.daemon_state.read().await;

    if daemon_state.status == ConnectionStatus::Online {
      ready_count += 1;
    }
  }

  if ready_count > 0 {
    (StatusCode::OK, "READY").into_response()
  } else {
    (StatusCode::SERVICE_UNAVAILABLE, "NOT READY").into_response()
  }
}

pub async fn get_namespace_health(
  Path(namespace): Path<Box<str>>,
  State(state): State<Arc<ServerState>>,
) -> Response {
  match state.upsd_servers.get(&namespace) {
    Some(upsd_state) => {
      let daemon_state = upsd_state.daemon_state.read().await;

      let response = Json(HealthResponse {
        last_device_sync: daemon_state.last_device_sync.as_ref(),
        upsd_port: upsd_state.config.port,
        upsd_server: &upsd_state.config.addr,
        upsd_status: daemon_state.status,
      });

      if response.upsd_status != ConnectionStatus::Dead {
        (StatusCode::OK, response).into_response()
      } else {
        (StatusCode::INTERNAL_SERVER_ERROR, response).into_response()
      }
    }
    None => (StatusCode::BAD_REQUEST).into_response(),
  }
}

pub async fn get_namespace_readiness(
  Path(namespace): Path<Box<str>>,
  State(state): State<Arc<ServerState>>,
) -> Response {
  match state.upsd_servers.get(&namespace) {
    Some(upsd_state) => {
      let daemon_state = upsd_state.daemon_state.read().await;

      if daemon_state.status == ConnectionStatus::Online {
        (StatusCode::OK, "READY").into_response()
      } else {
        (StatusCode::SERVICE_UNAVAILABLE, "NOT READY").into_response()
      }
    }
    None => (StatusCode::BAD_REQUEST, "UPSD NAMESPACE NOT EXIST").into_response(),
  }
}
