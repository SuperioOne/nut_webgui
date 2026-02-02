use crate::{
  http::json_api::{
    problem_detail::ProblemDetail,
    route::{extract_upsd, request_auth_client},
  },
  state::{ServerState, VarDetail},
};
use axum::{
  Json,
  extract::{
    Path, State,
    rejection::{JsonRejection, PathRejection},
  },
  http::StatusCode,
};
use nut_webgui_upsmc::{UpsName, Value, VarName};
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct RwRequest {
  variable: VarName,
  value: Value,
}

pub async fn patch(
  State(state): State<Arc<ServerState>>,
  paths: Result<Path<(Box<str>, UpsName)>, PathRejection>,
  body: Result<Json<RwRequest>, JsonRejection>,
) -> Result<StatusCode, ProblemDetail> {
  let Path((namespace, ups_name)) = paths?;
  let Json(body) = body?;
  let upsd = extract_upsd!(state, namespace.as_ref())?;

  {
    match upsd.daemon_state.read().await.devices.get(&ups_name) {
      Some(device) => match device.rw_variables.get(&body.variable) {
        Some(VarDetail::Number) => {
          if body.value.is_numeric() {
            Ok(())
          } else {
            Err(
              ProblemDetail::new("Invalid value type", StatusCode::BAD_REQUEST).with_detail(
                format!(
                  "'{var_name}' expects a numeric type, but the provided value is not a number.",
                  var_name = &body.variable
                ),
              ),
            )
          }
        }
        Some(VarDetail::String { max_len }) => {
          if body.value.is_text() {
            let value_str = body.value.as_str();
            let trimmed = value_str.trim();

            if trimmed.is_empty() {
              Err(
                ProblemDetail::new("Empty value", StatusCode::BAD_REQUEST)
                  .with_detail("Value cannot be empty or consist of only whitespaces.".to_owned()),
              )
            } else if trimmed.len() > *max_len {
              Err(
                ProblemDetail::new("Out of range", StatusCode::BAD_REQUEST)
                  .with_detail(format!("Maximum allowed string length is {}.", max_len)),
              )
            } else {
              Ok(())
            }
          } else {
            Err(
              ProblemDetail::new("Invalid value type", StatusCode::BAD_REQUEST).with_detail(
                format!(
                  "'{var_name}' expects a string type, but the provided value is not a string.",
                  var_name = &body.variable
                ),
              ),
            )
          }
        }
        Some(VarDetail::Enum { options }) => {
          if options.contains(&body.value) {
            Ok(())
          } else {
            Err(
              ProblemDetail::new("Invalid option", StatusCode::BAD_REQUEST).with_detail(format!(
                "'{var_name}' is an enum type, allowed options: {opts:?}",
                var_name = &body.variable,
                opts = options
                  .iter()
                  .map(|v| v.as_str())
                  .collect::<Vec<std::borrow::Cow<'_, str>>>()
              )),
            )
          }
        }
        Some(VarDetail::Range { min, max }) => {
          if body.value.is_numeric() {
            match (min.as_lossly_f64(), max.as_lossly_f64()) {
              (Some(min), Some(max)) => {
                let valuef64 = body.value.as_lossly_f64().unwrap_or(0.0);

                if min <= valuef64 && valuef64 <= max {
                  Ok(())
                } else {
                  Err(
                    ProblemDetail::new("Out of range", StatusCode::BAD_REQUEST).with_detail(
                      format!(
                        "'{var_name}' is not within the acceptable range [{min}, {max}]",
                        var_name = &body.variable,
                        min = min,
                        max = max,
                      ),
                    ),
                  )
                }
              }
              _ => Err(ProblemDetail::new("Malformed driver response", StatusCode::INTERNAL_SERVER_ERROR).with_detail(
                "Cannot process request since the reported min-max values by ups device are not number.".to_owned(),
              ),
            ),
            }
          } else {
            Err(
              ProblemDetail::new("Invalid value type", StatusCode::BAD_REQUEST).with_detail(
                format!(
                  "'{var_name}' expects a numeric value between {min} and {max}, but the provided value is not a number.",
                  var_name = &body.variable,
                  min = min,
                  max = max,
                ),
              ),
            )
          }
        }
        None => Err(
          ProblemDetail::new("Invalid RW variable", StatusCode::BAD_REQUEST).with_detail(format!(
            "'{var_name}' is not a valid writeable variable.",
            var_name = &body.variable
          )),
        ),
      },
      None => Err(ProblemDetail::new(
        "Device not found",
        StatusCode::NOT_FOUND,
      )),
    }
  }?;

  let mut client = request_auth_client!(upsd)?;

  {
    let response = client.set_var(&ups_name, &body.variable, &body.value).await;
    _ = client.close().await;

    response
  }?;

  info!(
    message = "set var request accepted",
    namespace = %namespace,
    device = %ups_name,
    variable = %body.variable,
    value = %body.value,
  );

  Ok(StatusCode::ACCEPTED)
}
