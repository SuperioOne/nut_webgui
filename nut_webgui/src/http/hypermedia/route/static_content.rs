use axum::{
  body::Body,
  http::{HeaderMap, HeaderValue, StatusCode, header},
  response::IntoResponse,
};

const CACHE_CONTROL_VALUE: &'static str = "private, max-age=604800, immutable";

pub async fn get_javascript() -> impl IntoResponse {
  let headers = HeaderMap::from_iter([
    (
      header::CONTENT_TYPE,
      HeaderValue::from_static(nut_webgui_client::JS.mime),
    ),
    (
      header::CACHE_CONTROL,
      HeaderValue::from_static(CACHE_CONTROL_VALUE),
    ),
  ]);

  (
    StatusCode::OK,
    headers,
    Body::from(nut_webgui_client::JS.bytes),
  )
}

pub async fn get_css() -> impl IntoResponse {
  let headers = HeaderMap::from_iter([
    (
      header::CONTENT_TYPE,
      HeaderValue::from_static(nut_webgui_client::CSS.mime),
    ),
    (
      header::CACHE_CONTROL,
      HeaderValue::from_static(CACHE_CONTROL_VALUE),
    ),
  ]);

  (
    StatusCode::OK,
    headers,
    Body::from(nut_webgui_client::CSS.bytes),
  )
}

pub async fn get_icon() -> impl IntoResponse {
  let headers = HeaderMap::from_iter([
    (
      header::CONTENT_TYPE,
      HeaderValue::from_static(nut_webgui_client::ICON.mime),
    ),
    (
      header::CACHE_CONTROL,
      HeaderValue::from_static(CACHE_CONTROL_VALUE),
    ),
  ]);

  (
    StatusCode::OK,
    headers,
    Body::from(nut_webgui_client::ICON.bytes),
  )
}

pub async fn get_sprite_sheet() -> impl IntoResponse {
  let headers = HeaderMap::from_iter([
    (
      header::CONTENT_TYPE,
      HeaderValue::from_static(nut_webgui_client::SPRITE_SHEET.mime),
    ),
    (
      header::CACHE_CONTROL,
      HeaderValue::from_static(CACHE_CONTROL_VALUE),
    ),
  ]);

  (
    StatusCode::OK,
    headers,
    Body::from(nut_webgui_client::SPRITE_SHEET.bytes),
  )
}
