use axum::{
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
      header::CONTENT_ENCODING,
      HeaderValue::from_static(nut_webgui_client::JS.encoding),
    ),
    (
      header::CACHE_CONTROL,
      HeaderValue::from_static(CACHE_CONTROL_VALUE),
    ),
  ]);

  (StatusCode::OK, headers, nut_webgui_client::JS.bytes)
}

pub async fn get_css() -> impl IntoResponse {
  let headers = HeaderMap::from_iter([
    (
      header::CONTENT_TYPE,
      HeaderValue::from_static(nut_webgui_client::CSS.mime),
    ),
    (
      header::CONTENT_ENCODING,
      HeaderValue::from_static(nut_webgui_client::CSS.encoding),
    ),
    (
      header::CACHE_CONTROL,
      HeaderValue::from_static(CACHE_CONTROL_VALUE),
    ),
  ]);

  (StatusCode::OK, headers, nut_webgui_client::CSS.bytes)
}

pub async fn get_icon() -> impl IntoResponse {
  let headers = HeaderMap::from_iter([
    (
      header::CONTENT_TYPE,
      HeaderValue::from_static(nut_webgui_client::ICON.mime),
    ),
    (
      header::CONTENT_ENCODING,
      HeaderValue::from_static(nut_webgui_client::ICON.encoding),
    ),
    (
      header::CACHE_CONTROL,
      HeaderValue::from_static(CACHE_CONTROL_VALUE),
    ),
  ]);

  (StatusCode::OK, headers, nut_webgui_client::ICON.bytes)
}

pub async fn get_sprite_sheet() -> impl IntoResponse {
  let headers = HeaderMap::from_iter([
    (
      header::CONTENT_TYPE,
      HeaderValue::from_static(nut_webgui_client::SPRITE_SHEET.mime),
    ),
    (
      header::CONTENT_ENCODING,
      HeaderValue::from_static(nut_webgui_client::SPRITE_SHEET.encoding),
    ),
    (
      header::CACHE_CONTROL,
      HeaderValue::from_static(CACHE_CONTROL_VALUE),
    ),
  ]);

  (
    StatusCode::OK,
    headers,
    nut_webgui_client::SPRITE_SHEET.bytes,
  )
}
