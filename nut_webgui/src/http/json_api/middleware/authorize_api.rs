use crate::{
  auth::{access_token::AccessToken, permission::Permissions},
  http::json_api::problem_detail::ProblemDetail,
};
use axum::{
  http::{HeaderValue, Request, StatusCode, header},
  response::{IntoResponse, Response},
};
use std::{future::Future, pin::Pin};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct AuthorizeApiLayer {
  permissions: Permissions,
}

impl AuthorizeApiLayer {
  pub fn new(permissions: Permissions) -> Self {
    Self { permissions }
  }
}

#[derive(Clone)]
pub struct AuthorizeApiService<S> {
  inner: S,
  permissions: Permissions,
}

impl<S> Layer<S> for AuthorizeApiLayer {
  type Service = AuthorizeApiService<S>;

  fn layer(&self, inner: S) -> Self::Service {
    Self::Service {
      inner,
      permissions: self.permissions,
    }
  }
}

impl<S, B> Service<Request<B>> for AuthorizeApiService<S>
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
    match req.extensions().get::<AccessToken>() {
      Some(access_token) => {
        if access_token.has_permission(self.permissions) {
          let inner_future = self.inner.call(req);

          Box::pin(inner_future)
        } else {
          let problem_detail = ProblemDetail::new("Access Denied", StatusCode::FORBIDDEN)
            .with_detail("Insufficient API permissions.".to_owned());

          Box::pin(async move { Ok(problem_detail.into_response()) })
        }
      }
      _ => {
        let problem_detail = ProblemDetail::new("Unauthorized", StatusCode::UNAUTHORIZED)
          .with_detail("Authorization required.".to_owned());

        Box::pin(async move {
          let mut response = problem_detail.into_response();
          let headers = response.headers_mut();

          headers.insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));

          Ok(response)
        })
      }
    }
  }
}
