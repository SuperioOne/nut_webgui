use crate::ups_daemon_state::{DaemonStatus, UpsDaemonState};
use axum::{
  http::{Request, StatusCode},
  response::{IntoResponse, Response},
};
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::RwLock;
use tower::{Layer, Service};

use super::common::ProblemDetailsResponse;

/// Checks daemon state and overrides http response if daemon is not `Online`.
#[derive(Clone)]
pub struct DaemonStateLayer {
  upsd_state: Arc<RwLock<UpsDaemonState>>,
}

impl DaemonStateLayer {
  pub fn new(upsd_state: Arc<RwLock<UpsDaemonState>>) -> Self {
    Self { upsd_state }
  }
}

#[derive(Clone)]
pub struct DaemonStateService<S> {
  inner: S,
  upsd_state: Arc<RwLock<UpsDaemonState>>,
}

impl<S> Layer<S> for DaemonStateLayer {
  type Service = DaemonStateService<S>;

  fn layer(&self, inner: S) -> Self::Service {
    Self::Service {
      inner,
      upsd_state: self.upsd_state.clone(),
    }
  }
}

impl<S, B> Service<Request<B>> for DaemonStateService<S>
where
  S: Service<Request<B>, Response = Response>,
  S::Future: Send + 'static,
{
  type Response = Response;
  type Error = S::Error;
  type Future = Pin<Box<dyn Send + Future<Output = Result<S::Response, S::Error>>>>;

  fn poll_ready(
    &mut self,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Result<(), Self::Error>> {
    self.inner.poll_ready(cx)
  }

  fn call(&mut self, req: Request<B>) -> Self::Future {
    let state = self.upsd_state.clone();
    let inner_future = self.inner.call(req);

    Box::pin(async move {
      let response = match state.read().await.status {
        DaemonStatus::Online => inner_future.await?,
        DaemonStatus::Dead => ProblemDetailsResponse {
          status: StatusCode::INTERNAL_SERVER_ERROR,
          title: "Ups daemon no connection",
          detail: Some(
            "Server is unable to connect upsd. See application logs for more details.".to_owned(),
          ),
          instance: None,
        }
        .into_response(),
        DaemonStatus::NotReady => ProblemDetailsResponse {
          status: StatusCode::SERVICE_UNAVAILABLE,
          title: "Server is not ready",
          detail: None,
          instance: None,
        }
        .into_response(),
      };

      Ok(response)
    })
  }
}
