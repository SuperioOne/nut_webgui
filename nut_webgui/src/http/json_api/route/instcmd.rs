use crate::http::{
  RouterState,
  json_api::{problem_detail::ProblemDetail, route::request_auth_client},
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
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct InstcmdRequest {
  instcmd: CmdName,
}

pub async fn post(
  State(rs): State<RouterState>,
  ups_name: Result<Path<UpsName>, PathRejection>,
  body: Result<Json<InstcmdRequest>, JsonRejection>,
) -> Result<StatusCode, ProblemDetail> {
  let Path(ups_name) = ups_name?;
  let Json(body) = body?;

  {
    let server_state = rs.state.read().await;

    match server_state.devices.get(&ups_name) {
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

  let mut client = request_auth_client!(rs)?;

  {
    let response = client.instcmd(&ups_name, &body.instcmd).await;
    _ = client.close().await;

    response
  }?;

  info!(
    message = "instcmd called",
    device = %ups_name,
    instcmd = %&body.instcmd
  );

  Ok(StatusCode::ACCEPTED)
}
