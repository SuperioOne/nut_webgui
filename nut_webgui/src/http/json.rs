use super::{RouterState, problem_detail::ProblemDetail};
use crate::{config::UpsdConfig, device_entry::DeviceEntry};
use axum::{
  Json,
  extract::{
    Path, State,
    rejection::{JsonRejection, PathRejection},
  },
  http::StatusCode,
  response::{IntoResponse, Response},
};
use nut_webgui_upsmc::{CmdName, UpsName, Value, VarName, clients::NutAuthClient};
use serde::Deserialize;
use tracing::{info, warn};

macro_rules! require_auth_config {
  ($config:expr) => {
    match $config {
      upsd @ UpsdConfig {
        pass: Some(pass),
        user: Some(user),
        ..
      } => Ok((upsd.get_socket_addr(), user.as_ref(), pass.as_ref())),
      _ => Err(
        ProblemDetail::new("Insufficient upsd configuration", StatusCode::UNAUTHORIZED)
          .with_detail("Operation requires valid username and password to be configured.".into()),
      ),
    }
  };
}

#[derive(Debug, Deserialize)]
pub struct CommandRequest {
  instcmd: CmdName,
}

#[derive(Debug, Deserialize)]
pub struct RwRequest {
  variable: VarName,
  value: Value,
}

pub async fn get_ups_by_name(
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

pub async fn get_ups_list(State(rs): State<RouterState>) -> Response {
  let server_state = rs.state.read().await;
  let mut device_refs: Vec<&DeviceEntry> = server_state.devices.values().collect();
  device_refs.sort_by(|r, l| r.name.cmp(&l.name));

  Json(device_refs).into_response()
}

pub async fn post_command(
  State(rs): State<RouterState>,
  ups_name: Result<Path<UpsName>, PathRejection>,
  body: Result<Json<CommandRequest>, JsonRejection>,
) -> Result<StatusCode, ProblemDetail> {
  let Path(ups_name) = ups_name?;
  let Json(body) = body?;
  let (addr, user, password) = require_auth_config!(&rs.config.upsd)?;

  _ = {
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

  let mut client = NutAuthClient::connect(addr, user, password).await?;

  _ = {
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

pub async fn post_fsd(
  State(rs): State<RouterState>,
  ups_name: Result<Path<UpsName>, PathRejection>,
) -> Result<StatusCode, ProblemDetail> {
  let Path(ups_name) = ups_name?;
  let (addr, user, password) = require_auth_config!(&rs.config.upsd)?;

  _ = {
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

  let mut client = NutAuthClient::connect(addr, user, password).await?;

  _ = {
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

pub async fn patch_var(
  State(rs): State<RouterState>,
  ups_name: Result<Path<UpsName>, PathRejection>,
  body: Result<Json<RwRequest>, JsonRejection>,
) -> Result<StatusCode, ProblemDetail> {
  let Path(ups_name) = ups_name?;
  let Json(body) = body?;
  let (addr, user, password) = require_auth_config!(&rs.config.upsd)?;

  _ = {
    let server_state = rs.state.read().await;

    // TODO: current implementations does not check if value type is correct
    match server_state.devices.get(&ups_name) {
      Some(device) => {
        if device.rw_variables.contains_key(&body.variable) {
          Ok(())
        } else {
          Err(
            ProblemDetail::new("Invalid RW variable", StatusCode::BAD_REQUEST).with_detail(
              format!(
                "'{var_name}' isn't listed as writeable on device details.",
                var_name = &body.variable
              ),
            ),
          )
        }
      }
      None => Err(ProblemDetail::new(
        "Device not found",
        StatusCode::NOT_FOUND,
      )),
    }
  }?;

  let mut client = NutAuthClient::connect(addr, user, password).await?;

  _ = {
    let response = client.set_var(&ups_name, &body.variable, &body.value).await;
    _ = client.close().await;

    response
  }?;

  info!(
    message = "set var called",
    device = %ups_name,
    variable = %body.variable,
    value = %body.value,
  );

  Ok(StatusCode::ACCEPTED)
}
