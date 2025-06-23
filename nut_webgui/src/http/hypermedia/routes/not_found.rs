use crate::http::{
  RouterState,
  hypermedia::{error::ErrorPage, utils::RenderWithConfig},
};
use askama::Template;
use axum::{
  extract::State,
  http::StatusCode,
  response::{Html, IntoResponse, Response},
};

#[derive(Template)]
#[template(path = "not_found/+page.html")]
struct NotFound;

pub async fn get(rs: State<RouterState>) -> Result<Response, ErrorPage<askama::Error>> {
  let response = (
    StatusCode::NOT_FOUND,
    Html(NotFound.render_with_config(&rs.config)?),
  )
    .into_response();

  Ok(response)
}
