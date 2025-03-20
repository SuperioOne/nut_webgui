use askama::Template;
use axum::response::{Html, IntoResponse, Response};

#[derive(Template)]
#[template(path = "not_found/+page.html")]
struct NotFound<'a> {
  title: &'a str,
}

pub async fn get() -> Response {
  let template = NotFound { title: "Not Found" };
  Html(template.render().unwrap()).into_response()
}
