use askama::Template;
use axum::{
  extract::State,
  response::{Html, IntoResponse, Response},
};

use crate::http::RouterState;

#[derive(Template)]
#[template(path = "themes.html")]
struct ThemesTemplate<'a> {
  default_theme: Option<&'a str>,
}

pub async fn get_themes(rs: State<RouterState>) -> Response {
  let template = ThemesTemplate {
    default_theme: rs.config.default_theme.as_deref(),
  };

  Html(template.render().unwrap()).into_response()
}
