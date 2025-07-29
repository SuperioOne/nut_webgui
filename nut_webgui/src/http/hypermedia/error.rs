use askama::Template;
use axum::{
  http::StatusCode,
  response::{Html, IntoResponse, Response},
};

#[derive(Template, Debug)]
#[template(path = "error.html")]
pub struct ErrorPage {
  message: String,
}

impl ErrorPage {
  #[inline]
  pub fn new(message: String) -> Self {
    Self { message }
  }
}

impl<E> From<E> for ErrorPage
where
  E: core::error::Error + Sized,
{
  #[inline]
  fn from(value: E) -> Self {
    Self {
      message: value.to_string(),
    }
  }
}

impl IntoResponse for ErrorPage {
  fn into_response(self) -> Response {
    match self.render() {
      Ok(html) => (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response(),
      Err(err) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Something went wrong {}", err),
      )
        .into_response(),
    }
  }
}
