use askama::Template;
use axum::response::{Html, IntoResponse, Response};

#[derive(Template)]
#[template(path = "not_found/+page.html")]
struct NotFound;

pub async fn get() -> Response {
  Html(NotFound.render().unwrap()).into_response()
}
