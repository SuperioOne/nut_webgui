use askama::Template;
use axum::response::{Html, IntoResponse, Response};

#[derive(Template)]
#[template(path = "themes.html")]
struct ThemesTemplate;

pub async fn get_themes() -> Response {
  let template = ThemesTemplate;
  Html(template.render().unwrap()).into_response()
}
