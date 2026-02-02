use crate::{
  auth::{
    AUTH_COOKIE_NAME, token_signer::TokenSigner, user_session::UserSession, user_store::UserStore,
  },
  config::ServerConfig,
};
use axum::{
  http::{HeaderValue, Request, header},
  response::Response,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use base64::{Engine, prelude::BASE64_STANDARD};
use std::{future::Future, pin::Pin, sync::Arc, time::Duration};
use tower::{Layer, Service};
use tracing::warn;

#[derive(Clone)]
pub struct RenewSessionLayer {
  config: Arc<ServerConfig>,
  user_store: Arc<UserStore>,
  renew_duration: Duration,
}

impl RenewSessionLayer {
  pub fn new(
    config: Arc<ServerConfig>,
    user_store: Arc<UserStore>,
    renew_duration: Duration,
  ) -> Self {
    Self {
      config,
      user_store,
      renew_duration,
    }
  }
}

#[derive(Clone)]
pub struct RenewSessionService<S> {
  inner: S,
  config: Arc<ServerConfig>,
  user_store: Arc<UserStore>,
  renew_duration: Duration,
}

impl<S> Layer<S> for RenewSessionLayer {
  type Service = RenewSessionService<S>;

  fn layer(&self, inner: S) -> Self::Service {
    Self::Service {
      inner,
      config: self.config.clone(),
      user_store: self.user_store.clone(),
      renew_duration: self.renew_duration,
    }
  }
}

impl<S, B> Service<Request<B>> for RenewSessionService<S>
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
    if let Some(session) = req.extensions_mut().get_mut::<UserSession>()
      && session.ttl() <= self.renew_duration
      && self.user_store.contains_user(session.get_username())
    {
      match self.user_store.renew_session(session.get_username()) {
        Ok(new_session) => {
          let signed_bytes =
            TokenSigner::new(self.config.server_key.as_bytes()).sign_token(&new_session);

          let ttl = new_session.ttl();
          let cookie = Cookie::build((AUTH_COOKIE_NAME, BASE64_STANDARD.encode(&signed_bytes)))
            .http_only(true)
            .same_site(SameSite::Strict)
            .path("/")
            .max_age(cookie::time::Duration::seconds(ttl.as_secs() as i64))
            .build()
            .to_string();

          let session_cookie = HeaderValue::from_str(&cookie.to_string())
            .expect("authentication cookie encoding is invalid");

          *session = new_session;

          let inner_call = self.inner.call(req);

          return Box::pin(async move {
            let mut response = inner_call.await?;
            let headers = response.headers_mut();

            headers.insert(header::SET_COOKIE, session_cookie);

            Ok(response)
          });
        }
        Err(err) => {
          warn!(message = "unable to renew user session", reason = %err, username = %session.get_username());
        }
      }
    }

    Box::pin(self.inner.call(req))
  }
}
