use crate::auth::{
  AUTH_COOKIE_NAME, signed_token::SignedToken, user_session::UserSession, user_store::UserStore,
};
use axum::{
  http::{HeaderMap, HeaderName, Request, StatusCode},
  response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use base64::{Engine, prelude::BASE64_STANDARD};
use std::{future::Future, pin::Pin, sync::Arc};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct UserAuthLayer {
  server_key: Arc<[u8]>,
  user_store: Arc<UserStore>,
  login_redirect: String,
}

impl UserAuthLayer {
  pub fn new(server_key: Arc<[u8]>, user_store: Arc<UserStore>, login_redirect: String) -> Self {
    Self {
      server_key,
      user_store,
      login_redirect,
    }
  }
}

#[derive(Clone)]
pub struct UserAuthService<S> {
  inner: S,
  server_key: Arc<[u8]>,
  user_store: Arc<UserStore>,
  login_redirect: String,
}

impl<S> Layer<S> for UserAuthLayer {
  type Service = UserAuthService<S>;

  fn layer(&self, inner: S) -> Self::Service {
    Self::Service {
      inner,
      login_redirect: self.login_redirect.clone(),
      server_key: self.server_key.clone(),
      user_store: self.user_store.clone(),
    }
  }
}

impl<S, B> Service<Request<B>> for UserAuthService<S>
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

    match self.extract_user_session(request_headers, &self.user_store) {
      Ok(session) => {
        let extensions = req.extensions_mut();
        extensions.insert(session);

        Box::pin(self.inner.call(req))
      }
      Err(_) => {
        let redirect = if request_headers.contains_key("HX-Request") {
          (
            StatusCode::OK,
            [(
              HeaderName::from_static("hx-redirect"),
              self.login_redirect.clone(),
            )],
          )
            .into_response()
        } else {
          Redirect::to(&self.login_redirect).into_response()
        };

        Box::pin(async move { Ok(redirect) })
      }
    }
  }
}

#[derive(Debug, Clone, Copy)]
struct InvalidCookieValue;

impl<S> UserAuthService<S> {
  fn extract_user_session(
    &self,
    headers: &HeaderMap,
    user_store: &UserStore,
  ) -> Result<UserSession, InvalidCookieValue> {
    let cookies = CookieJar::from_headers(headers);

    match cookies.get(AUTH_COOKIE_NAME) {
      Some(value) => {
        let value = value.value_trimmed();

        let bytes = BASE64_STANDARD
          .decode(value.as_bytes())
          .map_err(|_| InvalidCookieValue)?;

        let user_session: UserSession = SignedToken::new(&self.server_key)
          .from_bytes(&bytes)
          .map_err(|_| InvalidCookieValue)?;

        if user_session.is_active() && user_store.contains_user(user_session.get_username()) {
          Ok(user_session)
        } else {
          Err(InvalidCookieValue)
        }
      }
      None => Err(InvalidCookieValue),
    }
  }
}
