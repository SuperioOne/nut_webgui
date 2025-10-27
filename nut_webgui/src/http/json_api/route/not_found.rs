use crate::http::json_api::problem_detail::ProblemDetail;
use axum::{http::StatusCode, response::IntoResponse};

pub async fn get() -> impl IntoResponse {
  ProblemDetail::new("Route not found", StatusCode::NOT_FOUND)
}
