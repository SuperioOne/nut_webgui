use crate::{
  http::json_api::{
    problem_detail::ProblemDetail,
    route::{extract_upsd, request_auth_client},
  },
  state::ServerState,
};
use axum::{
  extract::{Path, State, rejection::PathRejection},
  http::StatusCode,
};
use nut_webgui_upsmc::UpsName;
use std::sync::Arc;
use tracing::warn;

pub async fn post(
  State(state): State<Arc<ServerState>>,
  Path(namespace): Path<Box<str>>,
  ups_name: Result<Path<UpsName>, PathRejection>,
) -> Result<StatusCode, ProblemDetail> {
  let Path(ups_name) = ups_name?;
  let upsd = extract_upsd!(state, namespace)?;

  {
    if upsd
      .daemon_state
      .read()
      .await
      .devices
      .contains_key(&ups_name)
    {
      Ok(())
    } else {
      Err(ProblemDetail::new(
        "Device not found",
        StatusCode::NOT_FOUND,
      ))
    }
  }?;

  let mut client = request_auth_client!(upsd)?;

  {
    let response = client.fsd(&ups_name).await;
    _ = client.close().await;

    response
  }?;

  warn!(
    message = "force shutdown (fsd) called",
    namespace = %namespace,
    device = %ups_name,
  );

  Ok(StatusCode::ACCEPTED)
}
