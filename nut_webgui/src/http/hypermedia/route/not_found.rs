use crate::{
  auth::user_session::UserSession,
  http::{
    RouterState,
    hypermedia::{error::ErrorPage, utils::RenderWithConfig},
  },
};
use askama::Template;
use axum::{
  Extension,
  extract::State,
  http::StatusCode,
  response::{Html, IntoResponse, Response},
};

#[derive(Template)]
#[template(path = "not_found/+page.html")]
struct NotFound;

pub async fn get(
  rs: State<RouterState>,
  session: Option<Extension<UserSession>>,
) -> Result<Response, ErrorPage> {
  let response = (
    StatusCode::NOT_FOUND,
    Html(NotFound.render_with_config(&rs.config, session.map(|v| v.0).as_ref())?),
  )
    .into_response();

  Ok(response)
}
