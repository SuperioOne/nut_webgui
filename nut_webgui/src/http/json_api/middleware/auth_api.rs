use crate::{
  auth::{access_token::AccessToken, token_signer::TokenSigner},
  config::ServerConfig,
  http::json_api::problem_detail::ProblemDetail,
};
use axum::{
  http::{HeaderMap, HeaderValue, Request, StatusCode, header},
  response::{IntoResponse, Response},
};
use base64::{Engine, prelude::BASE64_STANDARD};
use std::{future::Future, pin::Pin, sync::Arc};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct ApiAuthLayer {
  config: Arc<ServerConfig>,
}

impl ApiAuthLayer {
  pub fn new(config: Arc<ServerConfig>) -> Self {
    Self { config }
  }
}

#[derive(Clone)]
pub struct ApiAuthService<S> {
  inner: S,
  config: Arc<ServerConfig>,
}

impl<S> Layer<S> for ApiAuthLayer {
  type Service = ApiAuthService<S>;

  fn layer(&self, inner: S) -> Self::Service {
    Self::Service {
      inner,
      config: self.config.clone(),
    }
  }
}

impl<S, B> Service<Request<B>> for ApiAuthService<S>
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

  fn call(&mut self, mut req: Request<B>) -> Self::Future {
    let request_headers = req.headers();

    match self.extract_bearer_token(request_headers) {
      Ok(token) => {
        let extensions = req.extensions_mut();
        extensions.insert(token);

        let inner_future = self.inner.call(req);

        Box::pin(inner_future)
      }
      Err(_) => {
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

#[derive(Debug, Clone, Copy)]
struct InvalidAuthHeaderValue;

impl<S> ApiAuthService<S> {
  fn extract_bearer_token(
    &self,
    headers: &HeaderMap,
  ) -> Result<AccessToken, InvalidAuthHeaderValue> {
    let header_value = headers
      .get(header::AUTHORIZATION)
      .map_or(Err(InvalidAuthHeaderValue), |v| {
        v.to_str().map_err(|_| InvalidAuthHeaderValue)
      })?
      .trim();

    if header_value.is_empty() {
      return Err(InvalidAuthHeaderValue);
    }

    match header_value.split_once(|c: char| c.is_ascii_whitespace()) {
      Some((scheme, token)) => {
        if !scheme.trim().eq_ignore_ascii_case("bearer") {
          return Err(InvalidAuthHeaderValue);
        }

        let bytes = BASE64_STANDARD
          .decode(token.trim().as_bytes())
          .map_err(|_| InvalidAuthHeaderValue)?;

        let access_token: AccessToken = TokenSigner::new(self.config.server_key.as_bytes())
          .from_bytes(&bytes)
          .map_err(|_| InvalidAuthHeaderValue)?;

        if access_token.is_active() {
          Ok(access_token)
        } else {
          Err(InvalidAuthHeaderValue)
        }
      }
      None => Err(InvalidAuthHeaderValue),
    }
  }
}
