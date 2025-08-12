use crate::{
  auth::user_session::UserSession,
  config::UpsdConfig,
  device_entry::VarDetail,
  htmx_redirect, htmx_swap,
  http::{
    RouterState,
    hypermedia::{
      error::ErrorPage, notification::NotificationTemplate, route::ups::RwFormTemplate,
      semantic_type::SemanticType, utils::RenderWithConfig,
    },
  },
};
use axum::{
  Extension, Form,
  extract::{Path, State},
  http::StatusCode,
  response::{Html, IntoResponse, Response},
};
use nut_webgui_upsmc::{InferValueFrom, UpsName, Value, VarName};
use serde::Deserialize;
use std::time::Duration;
use tracing::{error, info};

#[derive(Deserialize, Debug)]
pub struct RwRequest {
  name: VarName,
  value: Box<str>,
}

enum ValidationResult {
  Valid { value: Value },
  Invalid { value: Value, message: &'static str },
}

fn validate_request(request_value: Box<str>, detail: &VarDetail) -> ValidationResult {
  match detail {
    VarDetail::String { max_len } => {
      if request_value.as_ref().trim().is_empty() {
        ValidationResult::Invalid {
          value: Value::from(request_value),
          message: "input is empty",
        }
      } else if request_value.as_ref().len() > *max_len {
        ValidationResult::Invalid {
          value: Value::from(request_value),
          message: "input is too long",
        }
      } else {
        ValidationResult::Valid {
          value: Value::from(request_value),
        }
      }
    }
    VarDetail::Number => match Value::infer_number_from(request_value.as_ref()) {
      Ok(value) => ValidationResult::Valid { value },
      Err(_) => ValidationResult::Invalid {
        value: Value::from(request_value),
        message: "input is not a number",
      },
    },
    VarDetail::Enum { options } => {
      let value = Value::from(request_value);

      if options.contains(&value) {
        ValidationResult::Valid { value }
      } else {
        ValidationResult::Invalid {
          value,
          message: "invalid option",
        }
      }
    }
    VarDetail::Range { min, max } => match Value::infer_number_from(request_value.as_ref()) {
      Ok(value) => match (min.as_lossly_f64(), max.as_lossly_f64()) {
        (Some(min), Some(max)) => {
          let valuef64 = value.as_lossly_f64().unwrap_or(0.0);

          if min <= valuef64 && valuef64 <= max {
            ValidationResult::Valid { value }
          } else {
            ValidationResult::Invalid {
              value: Value::from(request_value),
              message: "value is not in range",
            }
          }
        }
        _ => ValidationResult::Invalid {
          value: Value::from(request_value),
          message: "driver reported min-max values are not numeric values",
        },
      },
      Err(_) => ValidationResult::Invalid {
        value: Value::from(request_value),
        message: "input is not a number",
      },
    },
  }
}

pub async fn patch(
  State(rs): State<RouterState>,
  Path(ups_name): Path<UpsName>,
  session: Option<Extension<UserSession>>,
  Form(request): Form<RwRequest>,
) -> Result<Response, ErrorPage> {
  let session = session.map(|v| v.0);

  let (value, detail) = {
    let state = rs.state.read().await;

    let var_detail = match state.devices.get(&ups_name) {
      Some(device) => match device.rw_variables.get(&request.name) {
        Some(var_detail) => var_detail,
        None => {
          return Ok(htmx_swap!(
            Html(
              NotificationTemplate::from(format!(
                "device driver does not support write on {}",
                request.name
              ))
              .set_level(SemanticType::Error)
              .render_with_config(&rs.config, session.as_ref())?
            ),
            "none"
          ));
        }
      },
      None => {
        return Ok(
          htmx_redirect!(
            StatusCode::NOT_FOUND,
            format!("{}/not-found", rs.config.http_server.base_path)
          )
          .into_response(),
        );
      }
    };

    let value = match validate_request(request.value, &var_detail) {
      ValidationResult::Valid { value } => value,
      ValidationResult::Invalid { value, message } => {
        return Ok(
          Html(
            RwFormTemplate {
              value: Some(&value),
              semantic: SemanticType::Error,
              message: Some(message),
              detail: &var_detail,
              device_name: &ups_name,
              var_name: &request.name,
              notification: Some(
                NotificationTemplate::from("Input validation failed")
                  .set_level(SemanticType::Error),
              ),
            }
            .render_with_config(&rs.config, session.as_ref())?,
          )
          .into_response(),
        );
      }
    };

    // clone var detail data to drop lock guard soon as possible.
    (value, var_detail.clone())
  };

  let auth_client = match &rs.config.upsd {
    UpsdConfig {
      pass: Some(pass),
      user: Some(user),
      ..
    } => {
      let client = rs.connection_pool.get_client().await?;
      client.authenticate(user, pass).await
    }
    _ => {
      return Ok(htmx_swap!(
        Html(
          NotificationTemplate::from(
            "No username or password configured for UPS daemon. Server is in read-only mode.",
          )
          .render_with_config(&rs.config, session.as_ref())?
        ),
        "none"
      ));
    }
  };

  let response = match auth_client {
    Ok(mut auth_client) => {
      let result = auth_client.set_var(&ups_name, &request.name, &value).await;
      _ = auth_client.close().await;

      let (semantic, message, notification) = match result {
        Ok(_) => {
          info!(message = "set var request accepted", device = %ups_name, value = %value, name = %request.name);

          (
            SemanticType::Success,
            None,
            Some(
              NotificationTemplate::from("Set var request is accepted")
                .set_level(SemanticType::Success),
            ),
          )
        }
        Err(err) => {
          error!(message = "set var request failed", device = %ups_name,  value = %value, name = %request.name, reason = %err);

          (
            SemanticType::Error,
            Some("value is rejected by device driver"),
            Some(
              NotificationTemplate::from(format!("Set var request failed, {}", err))
                .set_level(SemanticType::Error)
                .set_ttl(Duration::from_secs(15)),
            ),
          )
        }
      };

      Html(
        RwFormTemplate {
          value: Some(&value),
          semantic,
          message,
          detail: &detail,
          device_name: &ups_name,
          var_name: &request.name,
          notification,
        }
        .render_with_config(&rs.config, session.as_ref())?,
      )
      .into_response()
    }
    Err(err) => {
      error!(message = "auth connection failed for set var request", value = %value, name = %request.name, reason = %err);

      htmx_swap!(
        Html(
          NotificationTemplate::from(format!("client authentication failed, {}", err))
            .set_level(SemanticType::Error)
            .render_with_config(&rs.config, session.as_ref())?,
        ),
        "none"
      )
    }
  };

  Ok(response)
}
