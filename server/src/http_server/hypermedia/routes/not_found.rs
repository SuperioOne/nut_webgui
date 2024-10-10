use askama::Template;
use axum::response::Response;
use axum_core::response::IntoResponse;

#[derive(Template)]
#[template(path = "not_found/+page.html")]
struct NotFound<'a> {
  title: &'a str,
}

pub async fn get() -> Response {
  let template = NotFound { title: "Not Found" };
  template.into_response()
}
