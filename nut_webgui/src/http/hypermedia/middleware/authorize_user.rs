use crate::{
  auth::{permission::Permissions, user_session::UserSession},
  config::ServerConfig,
  http::hypermedia::{
    error::ErrorPage, notification::NotificationTemplate, semantic_type::SemanticType,
    utils::RenderWithConfig,
  },
};
use axum::{
  http::{HeaderName, HeaderValue, Request, StatusCode},
  response::{Html, IntoResponse, Response},
};
use std::{future::Future, pin::Pin, sync::Arc};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct AuthorizeUserLayer {
  permissions: Permissions,
  server_config: Arc<ServerConfig>,
}

impl AuthorizeUserLayer {
  pub fn new(server_config: Arc<ServerConfig>, permissions: Permissions) -> Self {
    Self {
      permissions,
      server_config,
    }
  }
}

#[derive(Clone)]
pub struct AuthorizeUserService<S> {
  inner: S,
  permissions: Permissions,
  server_config: Arc<ServerConfig>,
}

impl<S> Layer<S> for AuthorizeUserLayer {
  type Service = AuthorizeUserService<S>;

  fn layer(&self, inner: S) -> Self::Service {
    Self::Service {
      inner,
      permissions: self.permissions,
      server_config: self.server_config.clone(),
    }
  }
}

impl<S, B> Service<Request<B>> for AuthorizeUserService<S>
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
    if let Some(session) = req.extensions().get::<UserSession>()
      && session.has_permission(self.permissions)
    {
      Box::pin(self.inner.call(req))
    } else if req.headers().contains_key("HX-Request") {
      let template = NotificationTemplate::new("User is not authorized for this action")
        .set_level(SemanticType::Error);

      let mut response = match template.render_with_config(&self.server_config, None) {
        Ok(html) => Html(html).into_response(),
        Err(err) => ErrorPage::from(err).into_response(),
      };

      let headers = response.headers_mut();
      headers.insert(
        HeaderName::from_static("hx-reswap"),
        HeaderValue::from_static("none"),
      );

      Box::pin(async move { Ok(response) })
    } else {
      Box::pin(async move { Ok((StatusCode::FORBIDDEN, "Forbidden").into_response()) })
    }
  }
}
