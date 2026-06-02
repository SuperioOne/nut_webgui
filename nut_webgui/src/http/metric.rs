use super::json_api::problem_detail::ProblemDetail;
use crate::state::ServerState;
use axum::{
  extract::State,
  http::{StatusCode, header::CONTENT_TYPE},
  response::{IntoResponse, Response},
};
use prometheus_client::encoding::text::encode;
use std::sync::Arc;
use tokio::task::spawn_blocking;

pub async fn get(State(state): State<Arc<ServerState>>) -> Result<Response, ProblemDetail> {
  // NOTE: prometheus_client::collector::Collector trait only allows blocking code, and
  // trying to convert already existing upsd state into openmetric format without data
  // duplication is mostly...meh... In the future, prometheus_client may be dropped in
  // favor of custom encoder.
  match spawn_blocking(move || {
    let mut buffer = String::new();
    encode(&mut buffer, &state.openmetrics).map(|_| buffer)
  })
  .await
  {
    Ok(Ok(stats)) => {
      let response = (
        StatusCode::OK,
        [(
          CONTENT_TYPE,
          "application/openmetrics-text; version=1.0.0; charset=utf-8",
        )],
        stats,
      )
        .into_response();

      Ok(response)
    }
    _ => Err(ProblemDetail::new(
      "Metric collection failed",
      StatusCode::INTERNAL_SERVER_ERROR,
    )),
  }
}
