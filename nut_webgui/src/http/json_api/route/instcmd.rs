use crate::{
  http::json_api::{
    problem_detail::ProblemDetail,
    route::{extract_upsd, request_auth_client},
  },
  state::ServerState,
};
use axum::{
  Json,
  extract::{
    Path, State,
    rejection::{JsonRejection, PathRejection},
  },
  http::StatusCode,
};
use nut_webgui_upsmc::{CmdName, UpsName};
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct InstcmdRequest {
  instcmd: CmdName,
}

pub async fn post(
  State(state): State<Arc<ServerState>>,
  paths: Result<Path<(Box<str>, UpsName)>, PathRejection>,
  body: Result<Json<InstcmdRequest>, JsonRejection>,
) -> Result<StatusCode, ProblemDetail> {
  let Path((namespace, ups_name)) = paths?;
  let Json(body) = body?;
  let upsd = extract_upsd!(state, namespace)?;

  {
    match upsd.daemon_state.read().await.devices.get(&ups_name) {
      Some(device) => {
        if device.commands.contains(&body.instcmd) {
          Ok(())
        } else {
          Err(
            ProblemDetail::new("Invalid INSTCMD", StatusCode::BAD_REQUEST).with_detail(format!(
              "'{cmd_name}' is not listed as supported command on device details.",
              cmd_name = &body.instcmd
            )),
          )
        }
      }
      None => Err(ProblemDetail::new(
        "Device not found",
        StatusCode::NOT_FOUND,
      )),
    }
  }?;

  let mut client = request_auth_client!(upsd)?;

  {
    let response = client.instcmd(&ups_name, &body.instcmd).await;
    _ = client.close().await;

    response
  }?;

  info!(
    message = "instcmd called",
    namespace = %namespace,
    device = %ups_name,
    instcmd = %&body.instcmd
  );

  Ok(StatusCode::ACCEPTED)
}
