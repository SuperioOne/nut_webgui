use crate::upsd_client::errors::NutClientErrors;
use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde::{ser::SerializeStruct, Serialize};

/// Generalized Http problem details response. It's based on RFC9457, but it doesn't
/// contain `type` field.
#[derive(Debug)]
pub struct ProblemDetailsResponse {
  pub title: &'static str,
  pub detail: Option<String>,
  pub instance: Option<String>,
  pub status: StatusCode,
}

impl Serialize for ProblemDetailsResponse {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut obj = serializer.serialize_struct("ProblemDetailsResponse", 4)?;
    obj.serialize_field("title", self.title)?;
    obj.serialize_field("detail", &self.detail)?;
    obj.serialize_field("instance", &self.instance)?;
    obj.serialize_field("status", &self.status.as_u16())?;
    obj.end()
  }
}

impl IntoResponse for ProblemDetailsResponse {
  fn into_response(self) -> Response {
    let status_code = self.status;
    let mut response = Json(self).into_response();
    let response_status = response.status_mut();

    *response_status = status_code;

    response
  }
}

impl From<NutClientErrors> for ProblemDetailsResponse {
  fn from(value: NutClientErrors) -> Self {
    match value {
      NutClientErrors::EmptyResponse => ProblemDetailsResponse {
        title: "UPS daemon empty response",
        instance: None,
        status: StatusCode::INTERNAL_SERVER_ERROR,
        detail: Some(
          "Server is able to connect daemon, but daemon isn't responding to any commands for now."
            .to_string(),
        ),
      },
      NutClientErrors::IOError { kind } => ProblemDetailsResponse {
        title: "UPS daemon IO error",
        instance: None,
        status: StatusCode::INTERNAL_SERVER_ERROR,
        detail: Some(kind.to_string()),
      },
      NutClientErrors::ParseError { kind } => ProblemDetailsResponse {
        title: "Invalid UPS daemon response",
        instance: None,
        status: StatusCode::INTERNAL_SERVER_ERROR,
        detail: Some(kind.to_string()),
      },
      NutClientErrors::ProtocolError { kind } => ProblemDetailsResponse {
        title: "UPS daemon protocol error",
        instance: None,
        status: StatusCode::INTERNAL_SERVER_ERROR,
        detail: Some(kind.to_string()),
      },
    }
  }
}
