use axum::{
  http::{HeaderMap, HeaderValue, StatusCode, header},
  response::IntoResponse,
};

pub async fn get_javascript() -> impl IntoResponse {
  let headers = HeaderMap::from_iter([
    (
      header::CONTENT_TYPE,
      HeaderValue::from_static("text/javascript"),
    ),
    (
      header::CACHE_CONTROL,
      HeaderValue::from_static("private, max-age=604800, immutable"),
    ),
  ]);

  (StatusCode::OK, headers, nut_webgui_client::JS.bytes)
}

pub async fn get_css() -> impl IntoResponse {
  let headers = HeaderMap::from_iter([
    (header::CONTENT_TYPE, HeaderValue::from_static("text/css")),
    (
      header::CACHE_CONTROL,
      HeaderValue::from_static("private, max-age=604800, immutable"),
    ),
  ]);

  (StatusCode::OK, headers, nut_webgui_client::CSS.bytes)
}

pub async fn get_icon() -> impl IntoResponse {
  let headers = HeaderMap::from_iter([
    (
      header::CONTENT_TYPE,
      HeaderValue::from_static("image/svg+xml"),
    ),
    (
      header::CACHE_CONTROL,
      HeaderValue::from_static("private, max-age=604800, immutable"),
    ),
  ]);

  (StatusCode::OK, headers, nut_webgui_client::ICON.bytes)
}

pub async fn get_sprite_sheet() -> impl IntoResponse {
  let headers = HeaderMap::from_iter([
    (
      header::CONTENT_TYPE,
      HeaderValue::from_static("image/svg+xml"),
    ),
    (
      header::CACHE_CONTROL,
      HeaderValue::from_static("private, max-age=604800, immutable"),
    ),
  ]);

  (
    StatusCode::OK,
    headers,
    nut_webgui_client::SPRITE_SHEET.bytes,
  )
}
