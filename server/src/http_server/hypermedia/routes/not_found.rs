use askama::Template;
use axum_core::response::IntoResponse;

#[derive(Template)]
#[template(path = "not_found/+page.html")]
struct NotFound<'a> {
  title: &'a str,
}

pub async fn get() -> impl IntoResponse {
  let content = NotFound { title: "Not Found" };
  content.into_response()
}