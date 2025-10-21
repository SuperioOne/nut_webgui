use axum::{
  http::{HeaderName, Request, StatusCode, header},
  response::Response,
};
use std::{future::Future, pin::Pin};
use tower::{Layer, Service};

/// HTMX can't interact with 3xx responses due to how XMLHttpRequest API works. This
/// middleware simply re-writes 3xx responses with HX-Redirect header when request is initiated by
/// HTMX (see HX-Request header)
#[derive(Clone)]
pub struct HtmxRedirectLayer;

impl HtmxRedirectLayer {
  #[inline]
  pub const fn new() -> Self {
    Self
  }
}

#[derive(Clone)]
pub struct HtmxRedirectService<S> {
  inner: S,
}

const HX_REDIRECT: HeaderName = HeaderName::from_static("hx-redirect");

impl<S> Layer<S> for HtmxRedirectLayer {
  type Service = HtmxRedirectService<S>;

  fn layer(&self, inner: S) -> Self::Service {
    Self::Service { inner }
  }
}

impl<S, B> Service<Request<B>> for HtmxRedirectService<S>
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
    match req.headers().get("HX-Request") {
      Some(_) => {
        let next = self.inner.call(req);

        Box::pin(async move {
          let mut response = next.await?;
          let status = response.status().as_u16();

          if status >= 300 && status < 400 {
            let response_headers = response.headers_mut();

            if let Some(location) = response_headers.get(header::LOCATION) {
              response_headers.insert(HX_REDIRECT, location.clone());
              *(response.status_mut()) = StatusCode::OK;
            };

            Ok(response)
          } else {
            Ok(response)
          }
        })
      }
      None => {
        let next = self.inner.call(req);
        Box::pin(async move { next.await })
      }
    }
  }
}
