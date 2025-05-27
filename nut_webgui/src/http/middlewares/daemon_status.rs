use crate::{
  http::problem_detail::ProblemDetail,
  state::{DaemonStatus, ServerState},
};
use axum::{
  http::{Request, StatusCode},
  response::{IntoResponse, Response},
};
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::RwLock;
use tower::{Layer, Service};

/// Checks daemon state and overrides http response if daemon is not `Online`.
#[derive(Clone)]
pub struct DaemonStateLayer {
  state: Arc<RwLock<ServerState>>,
}

impl DaemonStateLayer {
  pub fn new(upsd_state: Arc<RwLock<ServerState>>) -> Self {
    Self { state: upsd_state }
  }
}

#[derive(Clone)]
pub struct DaemonStateService<S> {
  inner: S,
  state: Arc<RwLock<ServerState>>,
}

impl<S> Layer<S> for DaemonStateLayer {
  type Service = DaemonStateService<S>;

  fn layer(&self, inner: S) -> Self::Service {
    Self::Service {
      inner,
      state: self.state.clone(),
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
    let state = self.state.clone();
    let inner_future = self.inner.call(req);

    Box::pin(async move {
      let upsd_status = { state.read().await.remote_state.status };

      let response = match upsd_status {
        DaemonStatus::Online => inner_future.await?,
        DaemonStatus::Dead => ProblemDetail {
          status: StatusCode::INTERNAL_SERVER_ERROR,
          title: "Ups daemon no connection",
          detail: Some(
            "Server is unable to connect upsd. See application logs for more details.".to_owned(),
          ),
        }
        .into_response(),
        DaemonStatus::NotReady => ProblemDetail {
          status: StatusCode::SERVICE_UNAVAILABLE,
          title: "Server is not ready",
          detail: None,
        }
        .into_response(),
      };

      Ok(response)
    })
  }
}
