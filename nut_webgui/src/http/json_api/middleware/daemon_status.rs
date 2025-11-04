use crate::{
  http::json_api::problem_detail::ProblemDetail,
  state::{ConnectionStatus, ServerState},
};
use axum::{
  extract::{OptionalFromRequestParts, Path, Request},
  http::StatusCode,
  response::{IntoResponse, Response},
};
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};
use tower::{Layer, Service};

/// Checks daemon state and overrides http response if daemon is not `Online`.
#[derive(Clone)]
pub struct DaemonStateLayer {
  state: Arc<ServerState>,
}

impl DaemonStateLayer {
  pub fn new(upsd_state: Arc<ServerState>) -> Self {
    Self { state: upsd_state }
  }
}

#[derive(Clone)]
pub struct DaemonStateService<S> {
  inner: S,
  state: Arc<ServerState>,
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

impl<S> Service<Request> for DaemonStateService<S>
where
  S: Service<Request, Response = Response> + Clone + Send + Sync + 'static,
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

  fn call(&mut self, req: Request) -> Self::Future {
    let state = self.state.clone();
    let (mut parts, body) = req.into_parts();
    let mut next: S = self.inner.clone();

    Box::pin(async move {
      let daemon_connection =
        match Path::<HashMap<String, String>>::from_request_parts(&mut parts, &state).await {
          Ok(Some(Path(parameters))) => match parameters.get("namespace") {
            Some(namespace) => match state.upsd_servers.get(namespace.as_str()) {
              Some(upsd) => match upsd.daemon_state.read().await.status {
                ConnectionStatus::Online => Ok(()),
                ConnectionStatus::Dead => Err(
                  ProblemDetail::new("No UPSD connection", StatusCode::SERVICE_UNAVAILABLE)
                    .with_detail(
                      "Server is unable to connect UPSD. See application logs for more details."
                        .into(),
                    ),
                ),
                ConnectionStatus::NotReady => Err(ProblemDetail::new(
                  "Upsd state is not ready",
                  StatusCode::SERVICE_UNAVAILABLE,
                )),
              },
              None => Ok(()),
            },
            None => Ok(()),
          },
          Ok(None) => Ok(()),
          Err(err) => Err(
            ProblemDetail::new("Unexpected path rejection", err.status())
              .with_detail(err.body_text()),
          ),
        };

      match daemon_connection {
        Ok(()) => {
          let req = Request::from_parts(parts, body);
          next.call(req).await
        }
        Err(problem_detail) => Ok(problem_detail.into_response()),
      }
    })
  }
}
