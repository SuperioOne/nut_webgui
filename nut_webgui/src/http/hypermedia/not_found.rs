use crate::{
  auth::user_session::UserSession, config::ServerConfig,
  http::hypermedia::util::RenderWithConfig as _,
};
use askama::Template;
use axum::response::IntoResponse;

#[derive(Template)]
#[template(path = "not_found/+page.html")]
pub struct NotFound;

impl NotFound {
  pub fn new_response(
    config: &ServerConfig,
    session: Option<&UserSession>,
  ) -> Result<impl IntoResponse, askama::Error> {
    Ok((
      axum::http::StatusCode::NOT_FOUND,
      [("HX-Retarget", "body")],
      axum::response::Html(Self.render_with_config(config, session)?),
    ))
  }
}
