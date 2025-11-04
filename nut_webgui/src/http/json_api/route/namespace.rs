use crate::{
  config::tls_mode::TlsMode,
  http::json_api::{problem_detail::ProblemDetail, route::extract_upsd},
  state::{ConnectionStatus, ServerState},
};
use axum::{
  Json,
  body::Bytes,
  extract::{Path, State},
  http::{StatusCode, header},
  response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::{
  Serialize,
  ser::{SerializeSeq, Serializer},
};
use std::sync::Arc;

#[derive(Serialize)]
struct UpsdEntry<'a> {
  pub address: &'a str,
  pub device_count: usize,
  pub last_sync_time: Option<&'a DateTime<Utc>>,
  pub namespace: &'a str,
  pub poll_freq: u64,
  pub poll_interval: u64,
  pub port: u16,
  pub protocol_version: Option<&'a str>,
  pub status: ConnectionStatus,
  pub tls_mode: TlsMode,
  pub version: Option<&'a str>,
}

pub async fn get(
  Path(namespace): Path<Box<str>>,
  State(state): State<Arc<ServerState>>,
) -> Result<Response, ProblemDetail> {
  let upsd = extract_upsd!(state, namespace)?;
  let daemon_state = upsd.daemon_state.read().await;

  let upsd_entry = UpsdEntry {
    address: &upsd.config.addr,
    device_count: daemon_state.devices.len(),
    last_sync_time: daemon_state.last_device_sync.as_ref(),
    namespace: &upsd.namespace,
    poll_freq: upsd.config.poll_freq,
    poll_interval: upsd.config.poll_interval,
    port: upsd.config.port,
    protocol_version: daemon_state.prot_ver.as_deref(),
    status: daemon_state.status,
    tls_mode: upsd.config.tls_mode,
    version: daemon_state.ver.as_deref(),
  };

  Ok(Json(upsd_entry).into_response())
}

pub async fn get_list(State(state): State<Arc<ServerState>>) -> Result<Response, ProblemDetail> {
  let mut serializer = serde_json::Serializer::new(Vec::new());
  let mut seq = serializer.serialize_seq(Some(state.upsd_servers.len()))?;

  for upsd in state.upsd_servers.values() {
    let daemon_state = upsd.daemon_state.read().await;

    let upsd_entry = UpsdEntry {
      address: &upsd.config.addr,
      device_count: daemon_state.devices.len(),
      last_sync_time: daemon_state.last_device_sync.as_ref(),
      namespace: &upsd.namespace,
      poll_freq: upsd.config.poll_freq,
      poll_interval: upsd.config.poll_interval,
      port: upsd.config.port,
      protocol_version: daemon_state.prot_ver.as_deref(),
      status: daemon_state.status,
      tls_mode: upsd.config.tls_mode,
      version: daemon_state.ver.as_deref(),
    };

    seq.serialize_element(&upsd_entry)?;
  }

  seq.end()?;

  let body = Bytes::from(serializer.into_inner());

  Ok(
    (
      StatusCode::OK,
      [(header::CONTENT_TYPE, "application/json")],
      body,
    )
      .into_response(),
  )
}
