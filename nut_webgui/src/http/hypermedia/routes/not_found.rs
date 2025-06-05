use askama::Template;
use axum::{
  extract::State,
  response::{Html, IntoResponse, Response},
};

use crate::http::RouterState;

#[derive(Template)]
#[template(path = "not_found/+page.html")]
struct NotFound<'a> {
  base_path: &'a str,
  default_theme: Option<&'a str>,
}

pub async fn get(rs: State<RouterState>) -> Response {
  let template = NotFound {
    base_path: rs.config.http_server.base_path.as_str(),
    default_theme: rs.config.default_theme.as_deref(),
  };

  Html(template.render().unwrap()).into_response()
}
