use crate::http::{
  RouterState,
  json_api::{problem_detail::ProblemDetail, route::request_auth_client},
};
use axum::{
  extract::{Path, State, rejection::PathRejection},
  http::StatusCode,
};
use nut_webgui_upsmc::UpsName;
use tracing::warn;

pub async fn post(
  State(rs): State<RouterState>,
  ups_name: Result<Path<UpsName>, PathRejection>,
) -> Result<StatusCode, ProblemDetail> {
  let Path(ups_name) = ups_name?;

  {
    let server_state = rs.state.read().await;

    if server_state.devices.contains_key(&ups_name) {
      Ok(())
    } else {
      Err(ProblemDetail::new(
        "Device not found",
        StatusCode::NOT_FOUND,
      ))
    }
  }?;

  let mut client = request_auth_client!(rs)?;

  {
    let response = client.fsd(&ups_name).await;
    _ = client.close().await;

    response
  }?;

  warn!(
    message = "force shutdown (fsd) called",
    device = %ups_name,
  );

  Ok(StatusCode::ACCEPTED)
}
