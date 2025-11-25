mod hypermedia;
mod json_api;
mod probe;

use crate::{
  auth::{AUTH_COOKIE_RENEW, permission::Permissions},
  http::{
    hypermedia::middleware::{
      auth_renew_session::RenewSessionLayer, auth_user::UserAuthLayer,
      authorize_user::AuthorizeUserLayer, htmx_redirect::HtmxRedirectLayer,
    },
    json_api::middleware::{
      auth_api::ApiAuthLayer, authorize_api::AuthorizeApiLayer, daemon_status::DaemonStateLayer,
      validate_content_length::ValidateEmptyContentLength,
    },
  },
  state::ServerState,
};
use axum::{
  Router, ServiceExt,
  http::{HeaderValue, StatusCode, header},
  routing::{get, patch, post},
};
use std::{sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tower::{Layer, ServiceBuilder};
use tower_http::{
  compression::CompressionLayer, cors::CorsLayer, limit::RequestBodyLimitLayer,
  normalize_path::NormalizePathLayer, set_header::SetResponseHeaderLayer, timeout::TimeoutLayer,
  trace::TraceLayer, validate_request::ValidateRequestHeaderLayer,
};

pub struct HttpServer {
  server_state: Arc<ServerState>,
}

impl HttpServer {
  pub fn new(server_state: Arc<ServerState>) -> Self {
    Self { server_state }
  }

  pub async fn serve<F>(self, listener: TcpListener, close_signal: F) -> Result<(), std::io::Error>
  where
    F: Future<Output = ()> + Send + 'static,
  {
    let Self { server_state } = self;
    let data_api = create_data_routes(server_state.clone());
    let hypermedia_api = create_hypermedia_routes(server_state.clone());

    let middleware = ServiceBuilder::new()
      .layer(TraceLayer::new_for_http())
      .layer(RequestBodyLimitLayer::new(65556)) // 64 MiB request payload limit
      .layer(SetResponseHeaderLayer::if_not_present(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, max-age=0"),
      ))
      .layer(TimeoutLayer::new(Duration::from_secs(30)))
      .layer(CompressionLayer::new().gzip(true).deflate(true));

    let probes = Router::new()
      .route("/health", get(probe::get_health))
      .route("/readiness", get(probe::get_readiness))
      .route("/health/{namespace}", get(probe::get_namespace_health))
      .route(
        "/readiness/{namespace}",
        get(probe::get_namespace_readiness),
      )
      .fallback(|| async { StatusCode::NOT_FOUND })
      .layer(CorsLayer::permissive());

    let router = Router::new()
      .nest("/api", data_api)
      .nest("/probes", probes)
      .merge(hypermedia_api)
      .layer(middleware)
      .with_state(server_state.clone());

    let router = if server_state.config.http_server.base_path.is_empty() {
      router.into_service()
    } else {
      Router::new()
        .nest(server_state.config.http_server.base_path.as_str(), router)
        .into_service()
    };

    let app = NormalizePathLayer::trim_trailing_slash().layer(router);

    axum::serve(listener, app.into_make_service())
      .with_graceful_shutdown(close_signal)
      .await
  }
}

#[inline]
fn create_data_routes(server_state: Arc<ServerState>) -> Router<Arc<ServerState>> {
  let data_api = Router::new()
    .route("/", get(json_api::route::namespace::get_list))
    .route("/{namespace}", get(json_api::route::namespace::get))
    .route("/{namespace}/devices", get(json_api::route::ups_list::get))
    .route(
      "/{namespace}/devices/{ups_name}",
      get(json_api::route::ups::get),
    )
    .route(
      "/{namespace}/devices/{ups_name}",
      patch(json_api::route::rw::patch).route_layer(
        ServiceBuilder::new().option_layer(
          server_state
            .auth_user_store
            .as_ref()
            .map(|_| AuthorizeApiLayer::new(Permissions::SETVAR)),
        ),
      ),
    )
    .route(
      "/{namespace}/devices/{ups_name}/instcmd",
      post(json_api::route::instcmd::post).route_layer(
        ServiceBuilder::new().option_layer(
          server_state
            .auth_user_store
            .as_ref()
            .map(|_| AuthorizeApiLayer::new(Permissions::INSTCMD)),
        ),
      ),
    )
    .route(
      "/{namespace}/devices/{ups_name}/fsd",
      post(json_api::route::fsd::post).route_layer(
        ServiceBuilder::new()
          .option_layer(
            server_state
              .auth_user_store
              .as_ref()
              .map(|_| AuthorizeApiLayer::new(Permissions::FSD)),
          )
          .layer(ValidateRequestHeaderLayer::custom(
            ValidateEmptyContentLength,
          )),
      ),
    )
    .fallback(json_api::route::not_found::get)
    .layer(
      ServiceBuilder::new()
        .layer(CorsLayer::permissive())
        .layer(ValidateRequestHeaderLayer::accept("application/json"))
        .option_layer(
          server_state
            .auth_user_store
            .as_ref()
            .map(|_| ApiAuthLayer::new(server_state.config.clone())),
        )
        .layer(DaemonStateLayer::new(server_state)),
    );

  data_api
}

#[inline]
fn create_hypermedia_routes(server_state: Arc<ServerState>) -> Router<Arc<ServerState>> {
  let static_files = Router::new()
    .route(
      "/style.css",
      get(hypermedia::route::static_content::get_css),
    )
    .route(
      "/index.js",
      get(hypermedia::route::static_content::get_javascript),
    )
    .route(
      "/icon.svg",
      get(hypermedia::route::static_content::get_icon),
    )
    .route(
      "/feather-sprite.svg",
      get(hypermedia::route::static_content::get_sprite_sheet),
    )
    .fallback(|| async { StatusCode::NOT_FOUND });

  let hypermedia_api = Router::new()
    .route("/", get(hypermedia::route::home::get))
    .route("/topology", get(hypermedia::route::topology::get))
    .route("/connection", get(hypermedia::route::connection::get))
    .route("/system", get(hypermedia::route::system::get))
    .route(
      "/ups/{namespace}/{ups_name}",
      get(hypermedia::route::ups::get),
    )
    .route(
      "/ups/{namespace}/{ups_name}/instcmd",
      post(hypermedia::route::ups::instcmd::post).route_layer(
        ServiceBuilder::new().option_layer(
          server_state
            .auth_user_store
            .as_ref()
            .map(|_| AuthorizeUserLayer::new(server_state.config.clone(), Permissions::INSTCMD)),
        ),
      ),
    )
    .route(
      "/ups/{namespace}/{ups_name}/fsd",
      post(hypermedia::route::ups::fsd::post).route_layer(
        ServiceBuilder::new().option_layer(
          server_state
            .auth_user_store
            .as_ref()
            .map(|_| AuthorizeUserLayer::new(server_state.config.clone(), Permissions::FSD)),
        ),
      ),
    )
    .route(
      "/ups/{namespace}/{ups_name}/rw",
      patch(hypermedia::route::ups::rw::patch).route_layer(
        ServiceBuilder::new().option_layer(
          server_state
            .auth_user_store
            .as_ref()
            .map(|_| AuthorizeUserLayer::new(server_state.config.clone(), Permissions::SETVAR)),
        ),
      ),
    )
    .route("/not-found", get(hypermedia::route::not_found::get))
    .fallback(hypermedia::route::not_found::get);

  match server_state.auth_user_store.as_ref() {
    Some(store) => hypermedia_api
      .layer(RenewSessionLayer::new(
        server_state.config.clone(),
        store.clone(),
        AUTH_COOKIE_RENEW,
      ))
      .route("/logout", post(hypermedia::route::logout::post))
      .route(
        "/api-keys",
        get(hypermedia::route::api_key::get).post(hypermedia::route::api_key::post),
      )
      .layer(UserAuthLayer::new(
        server_state.config.clone(),
        store.clone(),
        format!("{}/login", server_state.config.http_server.base_path),
      ))
      .route(
        "/login",
        get(hypermedia::route::login::get).post(hypermedia::route::login::post),
      ),
    _ => hypermedia_api,
  }
  .route(
    "/_layout/themes",
    get(hypermedia::route::layout::get_themes),
  )
  .layer(HtmxRedirectLayer::new())
  .nest_service("/static", static_files)
}
