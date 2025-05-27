use axum::{
  body::Body,
  http::{HeaderValue, StatusCode},
  response::IntoResponse,
};
use tower_http::validate_request::ValidateRequest;

#[derive(Clone, Copy)]
pub struct ValidateEmptyContentLength;

const ZERO_LEN: HeaderValue = HeaderValue::from_static("0");

impl<B> ValidateRequest<B> for ValidateEmptyContentLength {
  type ResponseBody = Body;

  fn validate(
    &mut self,
    request: &mut axum::http::Request<B>,
  ) -> Result<(), axum::http::Response<Self::ResponseBody>> {
    match request.headers().get(axum::http::header::CONTENT_LENGTH) {
      Some(header) if header.eq(&ZERO_LEN) => Ok(()),
      Some(_) => Err(StatusCode::NOT_ACCEPTABLE.into_response()),
      None => Ok(()),
    }
  }
}
