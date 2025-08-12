use crate::http::{RouterState, json_api::problem_detail::ProblemDetail};
use axum::{
  Json,
  extract::{Path, State, rejection::PathRejection},
  http::StatusCode,
  response::{IntoResponse, Response},
};
use nut_webgui_upsmc::UpsName;

pub async fn get(
  State(rs): State<RouterState>,
  ups_name: Result<Path<UpsName>, PathRejection>,
) -> Result<Response, ProblemDetail> {
  let Path(ups_name) = ups_name?;
  let server_state = rs.state.read().await;

  if let Some(ups) = server_state.devices.get(&ups_name) {
    Ok(Json(ups).into_response())
  } else {
    Err(ProblemDetail::new(
      "Device not found",
      StatusCode::NOT_FOUND,
    ))
  }
}
