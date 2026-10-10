use axum::{
  Json,
  extract::rejection::{JsonRejection, PathRejection},
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde::{Serialize, ser::SerializeStruct};
use upsmc::error::{Error, ErrorKind};

pub trait IntoDetailStr {
  fn into_detail_str(self) -> String;
}

/// Generalized Http problem details response. It's based on RFC9457, but it
/// doesn't contain `type` field.
#[derive(Debug)]
pub struct ProblemDetail {
  pub title: &'static str,
  pub detail: Option<String>,
  pub status: StatusCode,
}

impl ProblemDetail {
  #[inline]
  pub const fn new(title: &'static str, status: StatusCode) -> Self {
    Self {
      title,
      detail: None,
      status,
    }
  }

  #[inline]
  pub fn with_detail<S>(mut self, detail: S) -> Self
  where
    S: IntoDetailStr,
  {
    self.detail = Some(detail.into_detail_str());
    self
  }

  #[inline]
  pub fn with_detail_error<E>(self, err: E) -> Self
  where
    E: core::error::Error,
  {
    self.with_detail(err.to_string())
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
    let title = match err.kind() {
      ErrorKind::EmptyResponse => "UPSD empty response",
      ErrorKind::IOError { .. } => "UPSD IO error",
      ErrorKind::ParseError { .. } => "Invalid UPSD response",
      ErrorKind::ProtocolError { .. } => "UPSD protocol error",
      ErrorKind::ConnectionPoolClosed => "UPSD connection pool error",
      ErrorKind::RequestTimeout => "Request timeout",
    };

    ProblemDetail::new(title, StatusCode::INTERNAL_SERVER_ERROR).with_detail_error(err)
  }
}

impl From<PathRejection> for ProblemDetail {
  fn from(err: PathRejection) -> Self {
    let title = match err {
      PathRejection::FailedToDeserializePathParams(_) => "Unable to deserialize path parameter",
      PathRejection::MissingPathParams(_) => "Missing path parameter",
      _ => "Invalid path",
    };

    ProblemDetail::new(title, err.status()).with_detail_error(err)
  }
}

impl From<JsonRejection> for ProblemDetail {
  fn from(err: JsonRejection) -> Self {
    let title = match err {
      JsonRejection::JsonDataError(_) => "Json data error",
      JsonRejection::JsonSyntaxError(_) => "Json syntax error",
      JsonRejection::MissingJsonContentType(_) => "Missing content type",
      JsonRejection::BytesRejection(_) => "Bytes rejection",
      _ => "Json content issue",
    };

    ProblemDetail::new(title, err.status()).with_detail_error(err)
  }
}

impl From<serde_json::error::Error> for ProblemDetail {
  fn from(err: serde_json::error::Error) -> Self {
    ProblemDetail::new("Json error", StatusCode::INTERNAL_SERVER_ERROR).with_detail_error(err)
  }
}

impl IntoDetailStr for &str {
  #[inline]
  fn into_detail_str(self) -> String {
    self.to_owned()
  }
}

impl IntoDetailStr for String {
  #[inline]
  fn into_detail_str(self) -> String {
    self
  }
}
