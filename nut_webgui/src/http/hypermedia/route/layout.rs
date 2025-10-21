use crate::http::{
  ServerState,
  hypermedia::{error::ErrorPage, utils::RenderWithConfig},
};
use askama::Template;
use axum::{
  extract::State,
  response::{Html, IntoResponse, Response},
};
use std::sync::Arc;

#[derive(Template)]
#[template(path = "themes.html")]
struct ThemesTemplate<'a> {
  default_theme: Option<&'a str>,
}

pub async fn get_themes(state: State<Arc<ServerState>>) -> Result<Response, ErrorPage> {
  let template = ThemesTemplate {
    default_theme: state.config.default_theme.as_deref(),
  };

  Ok(Html(template.render_with_config(&state.config, None)?).into_response())
}
