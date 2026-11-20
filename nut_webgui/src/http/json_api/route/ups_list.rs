use crate::{
  http::json_api::{problem_detail::ProblemDetail, route::extract_upsd},
  state::{DeviceEntry, ServerState},
};
use axum::{
  Json,
  extract::{Path, State},
  response::{IntoResponse, Response},
};
use serde::ser::SerializeSeq;
use std::{collections::BTreeMap, sync::Arc};
use upsmc::UpsName;

// NOTE: Wrapper type to implement Serialize trait over values() iterator.
struct DeviceList<'a> {
  list: &'a BTreeMap<UpsName, DeviceEntry>,
}

pub async fn get(
  State(state): State<Arc<ServerState>>,
  Path(namespace): Path<Box<str>>,
) -> Result<Response, ProblemDetail> {
  let upsd = extract_upsd!(state, namespace.as_ref())?;
  let daemon_state = upsd.daemon_state.read().await;

  Ok(
    Json(DeviceList {
      list: &daemon_state.devices,
    })
    .into_response(),
  )
}

impl serde::Serialize for DeviceList<'_> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut list = serializer.serialize_seq(Some(self.list.len()))?;

    for entry in self.list.values() {
      list.serialize_element(&entry)?;
    }

    list.end()
  }
}
