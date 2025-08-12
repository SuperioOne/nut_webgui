use crate::{device_entry::DeviceEntry, http::RouterState};
use axum::{
  Json,
  extract::State,
  response::{IntoResponse, Response},
};

pub async fn get(State(rs): State<RouterState>) -> Response {
  let server_state = rs.state.read().await;
  let mut device_refs: Vec<&DeviceEntry> = server_state.devices.values().collect();
  device_refs.sort_by(|r, l| r.name.cmp(&l.name));

  Json(device_refs).into_response()
}
