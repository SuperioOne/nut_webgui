use askama::Template;
use axum::{
  http::StatusCode,
  response::{Html, IntoResponse, Response},
};

#[derive(Template, Debug)]
#[template(path = "error.html")]
pub struct ErrorPage<E>
where
  E: core::error::Error,
{
  message: E,
}

impl<E> From<E> for ErrorPage<E>
where
  E: core::error::Error,
{
  #[inline]
  fn from(value: E) -> Self {
    Self { message: value }
  }
}

impl<E> IntoResponse for ErrorPage<E>
where
  E: core::error::Error,
{
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
