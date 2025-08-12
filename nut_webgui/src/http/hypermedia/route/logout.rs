use crate::{
  auth::AUTH_COOKIE_NAME,
  http::{RouterState, hypermedia::error::ErrorPage},
};
use axum::{
  extract::State,
  http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header},
  response::{IntoResponse, Response},
};
use cookie::{Cookie, SameSite, time::OffsetDateTime};

pub async fn post(rs: State<RouterState>) -> Result<Response, ErrorPage> {
  let redirect_path = format!("{}/", rs.config.http_server.base_path);

  let cookie = Cookie::build((AUTH_COOKIE_NAME, ""))
    .http_only(true)
    .same_site(SameSite::Strict)
    .path("/")
    .expires(OffsetDateTime::from_unix_timestamp(0)?)
    .build();

  let mut headers = HeaderMap::new();

  headers.insert(
    header::SET_COOKIE,
    HeaderValue::from_str(&cookie.to_string())?,
  );
  headers.insert(
    HeaderName::from_static("hx-redirect"),
    HeaderValue::from_str(&redirect_path)?,
  );

  Ok((StatusCode::OK, headers).into_response())
}
