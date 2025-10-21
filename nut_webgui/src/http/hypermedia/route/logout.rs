use crate::{auth::AUTH_COOKIE_NAME, http::hypermedia::error::ErrorPage, state::ServerState};
use axum::{
  extract::State,
  http::{HeaderValue, header},
  response::{IntoResponse, Redirect, Response},
};
use cookie::{Cookie, SameSite, time::OffsetDateTime};
use std::sync::Arc;

pub async fn post(state: State<Arc<ServerState>>) -> Result<Response, ErrorPage> {
  let redirect_path = format!("{}/", state.config.http_server.base_path);

  let cookie = Cookie::build((AUTH_COOKIE_NAME, ""))
    .http_only(true)
    .same_site(SameSite::Strict)
    .path("/")
    .expires(OffsetDateTime::from_unix_timestamp(0)?)
    .build();

  let mut response = Redirect::to(&redirect_path).into_response();

  response.headers_mut().insert(
    header::SET_COOKIE,
    HeaderValue::from_str(&cookie.to_string())?,
  );

  Ok(response)
}
