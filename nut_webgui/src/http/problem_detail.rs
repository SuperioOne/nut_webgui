use axum::{
  Json,
  extract::rejection::{JsonRejection, PathRejection},
  http::StatusCode,
  response::{IntoResponse, Response},
};
use nut_webgui_upsmc::errors::{Error, ErrorKind};
use serde::{Serialize, ser::SerializeStruct};

/// Generalized Http problem details response. It's based on RFC9457, but it doesn't
/// contain `type` field.
#[derive(Debug)]
pub struct ProblemDetail {
  pub title: &'static str,
  pub detail: Option<String>,
  pub status: StatusCode,
}

impl ProblemDetail {
  pub fn new(title: &'static str, status: StatusCode) -> Self {
    Self {
      title,
      detail: None,
      status,
    }
  }

  #[inline]
  pub fn with_detail(mut self, detail: String) -> Self {
    self.detail = Some(detail);
    self
  }
}

impl Serialize for ProblemDetail {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut obj = serializer.serialize_struct("ProblemDetailsResponse", 3)?;
    obj.serialize_field("title", self.title)?;
    obj.serialize_field("detail", &self.detail)?;
    obj.serialize_field("status", &self.status.as_u16())?;
    obj.end()
  }
}

impl IntoResponse for ProblemDetail {
  fn into_response(self) -> Response {
    let status_code = self.status;
    let mut response = Json(self).into_response();
    let response_status = response.status_mut();

    *response_status = status_code;

    response
  }
}

impl From<Error> for ProblemDetail {
  fn from(err: Error) -> Self {
    match err.kind() {
      ErrorKind::EmptyResponse => ProblemDetail {
        title: "UPS daemon empty response",
        status: StatusCode::INTERNAL_SERVER_ERROR,
        detail: Some(
          "Server is able to connect daemon, but daemon isn't responding to any commands for now."
            .to_string(),
        ),
      },
      ErrorKind::IOError { .. } => ProblemDetail {
        title: "UPS daemon IO error",
        status: StatusCode::INTERNAL_SERVER_ERROR,
        detail: Some(err.to_string()),
      },
      ErrorKind::ParseError { .. } => ProblemDetail {
        title: "Invalid UPS daemon response",
        status: StatusCode::INTERNAL_SERVER_ERROR,
        detail: Some(err.to_string()),
      },
      ErrorKind::ProtocolError { .. } => ProblemDetail {
        title: "UPS daemon protocol error",
        status: StatusCode::INTERNAL_SERVER_ERROR,
        detail: Some(err.to_string()),
      },
      ErrorKind::ConnectionPoolClosed => ProblemDetail {
        title: "Server connection pool error",
        status: StatusCode::INTERNAL_SERVER_ERROR,
        detail: Some(err.to_string()),
      },
      ErrorKind::RequestTimeout => ProblemDetail {
        title: "Request timeout",
        status: StatusCode::INTERNAL_SERVER_ERROR,
        detail: Some(err.to_string()),
      },
    }
  }
}

impl From<PathRejection> for ProblemDetail {
  fn from(value: PathRejection) -> Self {
    match value {
      PathRejection::FailedToDeserializePathParams(err) => ProblemDetail {
        title: "Unable to deserialize path parameter",
        detail: Some(err.body_text()),
        status: err.status(),
      },
      PathRejection::MissingPathParams(err) => ProblemDetail {
        title: "Missing path parameter",
        detail: Some(err.body_text()),
        status: err.status(),
      },
      c => ProblemDetail {
        title: "Invalid path",
        detail: Some(c.body_text()),
        status: c.status(),
      },
    }
  }
}

impl From<JsonRejection> for ProblemDetail {
  fn from(value: JsonRejection) -> Self {
    match value {
      JsonRejection::JsonDataError(err) => ProblemDetail {
        title: "Json data error",
        detail: Some(err.body_text()),
        status: err.status(),
      },
      JsonRejection::JsonSyntaxError(err) => ProblemDetail {
        title: "Json syntax error",
        detail: Some(err.body_text()),
        status: err.status(),
      },
      JsonRejection::MissingJsonContentType(err) => ProblemDetail {
        title: "Missing content type",
        detail: Some(err.body_text()),
        status: err.status(),
      },
      JsonRejection::BytesRejection(err) => ProblemDetail {
        title: "Bytes rejection",
        detail: Some(err.body_text()),
        status: err.status(),
      },
      c => ProblemDetail {
        title: "Json content issue",
        detail: Some(c.body_text()),
        status: c.status(),
      },
    }
  }
}
