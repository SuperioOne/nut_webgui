use crate::state::{ConnectionStatus, ServerState};
use axum::{
  extract::{Path, State},
  http::StatusCode,
  response::{IntoResponse, Response},
};
use std::sync::Arc;

pub enum HealthStatus {
  Degraded,
  Ok,
  Dead,
}

pub enum Readiness {
  Ready,
  NotReady,
}

impl IntoResponse for HealthStatus {
  fn into_response(self) -> Response {
    match self {
      HealthStatus::Degraded => (StatusCode::OK, "DEGRADED"),
      HealthStatus::Ok => (StatusCode::OK, "OK"),
      HealthStatus::Dead => (StatusCode::INTERNAL_SERVER_ERROR, "DEAD"),
    }
    .into_response()
  }
}

impl IntoResponse for Readiness {
  fn into_response(self) -> Response {
    match self {
      Readiness::Ready => (StatusCode::OK, "READY"),
      Readiness::NotReady => (StatusCode::SERVICE_UNAVAILABLE, "NOT-READY"),
    }
    .into_response()
  }
}

pub async fn get_health(State(state): State<Arc<ServerState>>) -> HealthStatus {
  let mut active_count: usize = 0;

  for upsd in state.upsd_servers.values() {
    let daemon_state = upsd.daemon_state.read().await;

    if daemon_state.status != ConnectionStatus::Dead {
      active_count += 1;
    }
  }

  if active_count == state.upsd_servers.len() {
    HealthStatus::Ok
  } else if active_count > 0 {
    HealthStatus::Degraded
  } else {
    HealthStatus::Dead
  }
}

pub async fn get_readiness(State(state): State<Arc<ServerState>>) -> Readiness {
  for upsd in state.upsd_servers.values() {
    let daemon_state = upsd.daemon_state.read().await;

    if daemon_state.status == ConnectionStatus::Online {
      return Readiness::Ready;
    }
  }

  Readiness::NotReady
}

pub async fn get_namespace_health(
  Path(namespace): Path<Box<str>>,
  State(state): State<Arc<ServerState>>,
) -> Response {
  match state.upsd_servers.get(namespace.as_ref()) {
    Some(upsd_state) => {
      let daemon_state = upsd_state.daemon_state.read().await;

      let status = if daemon_state.status != ConnectionStatus::Dead {
        HealthStatus::Ok
      } else {
        HealthStatus::Dead
      };

      status.into_response()
    }
    None => (StatusCode::NOT_FOUND, "UPSD NAMESPACE DOES NOT EXIST").into_response(),
  }
}

pub async fn get_namespace_readiness(
  Path(namespace): Path<Box<str>>,
  State(state): State<Arc<ServerState>>,
) -> Response {
  match state.upsd_servers.get(namespace.as_ref()) {
    Some(upsd_state) => {
      let daemon_state = upsd_state.daemon_state.read().await;

      let status = if daemon_state.status == ConnectionStatus::Online {
        Readiness::Ready
      } else {
        Readiness::NotReady
      };

      status.into_response()
    }
    None => (StatusCode::NOT_FOUND, "UPSD NAMESPACE DOES NOT EXIST").into_response(),
  }
}
