use crate::http::{
  RouterState,
  hypermedia::{error::ErrorPage, utils::RenderWithConfig},
};
use askama::Template;
use axum::{
  extract::State,
  response::{Html, IntoResponse, Response},
};

#[derive(Template)]
#[template(path = "themes.html")]
struct ThemesTemplate<'a> {
  default_theme: Option<&'a str>,
}

pub async fn get_themes(rs: State<RouterState>) -> Result<Response, ErrorPage> {
  let template = ThemesTemplate {
    default_theme: rs.config.default_theme.as_deref(),
  };

  Ok(Html(template.render_with_config(&rs.config)?).into_response())
}
