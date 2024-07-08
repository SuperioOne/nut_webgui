use askama::Template;
use axum_core::response::IntoResponse;

#[derive(Template)]
#[template(path = "not_found/+page.html")]
struct NotFound<'a> {
  title: &'a str,
}

pub async fn get() -> impl IntoResponse {
  let template = NotFound { title: "Not Found" };
  template.into_response()
}

