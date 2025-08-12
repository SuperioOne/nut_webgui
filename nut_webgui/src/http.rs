mod hypermedia;
mod json_api;
mod probe;

use crate::{
  auth::{AUTH_COOKIE_RENEW, permission::Permissions, user_store::UserStore},
  config::ServerConfig,
  http::{
    hypermedia::middleware::{
      auth_renew_session::RenewSessionLayer, auth_user::UserAuthLayer,
      authorize_user::AuthorizeUserLayer,
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
use nut_webgui_upsmc::client::NutPoolClient;
use std::{sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::RwLock};
use tower::{Layer, ServiceBuilder};
use tower_http::{
  compression::CompressionLayer, cors::CorsLayer, limit::RequestBodyLimitLayer,
  normalize_path::NormalizePathLayer, set_header::SetResponseHeaderLayer, timeout::TimeoutLayer,
  trace::TraceLayer, validate_request::ValidateRequestHeaderLayer,
};

#[derive(Clone)]
struct RouterState {
  config: Arc<ServerConfig>,
  state: Arc<RwLock<ServerState>>,
  connection_pool: NutPoolClient,
  auth_user_store: Option<Arc<UserStore>>,
}

pub struct HttpServer {
  config: ServerConfig,
  server_state: Arc<RwLock<ServerState>>,
  connection_pool: NutPoolClient,
  auth_user_store: Option<Arc<UserStore>>,
}

impl HttpServer {
  pub fn new(
    config: ServerConfig,
    server_state: Arc<RwLock<ServerState>>,
    connection_pool: NutPoolClient,
  ) -> Self {
    Self {
      config,
      server_state,
      connection_pool,
      auth_user_store: None,
    }
  }

  #[inline]
  pub fn set_auth(&mut self, store: Arc<UserStore>) {
    self.auth_user_store = Some(store);
  }

  pub async fn serve<F>(self, listener: TcpListener, close_signal: F) -> Result<(), std::io::Error>
  where
    F: Future<Output = ()> + Send + 'static,
  {
    let Self {
      server_state,
      config,
      connection_pool,
      auth_user_store,
    } = self;

    let server_key: Arc<[u8]> = Arc::from(config.server_key.as_bytes());
    let shared_config = Arc::new(config);

    let data_api = create_data_routes(&auth_user_store, server_key.clone(), server_state.clone());
    let hypermedia_api =
      create_hypermedia_routes(&auth_user_store, server_key, shared_config.clone());

    let router_state = RouterState {
      auth_user_store,
      config: shared_config.clone(),
      state: server_state,
      connection_pool,
    };

    let middleware = ServiceBuilder::new()
      .layer(TraceLayer::new_for_http())
      .layer(RequestBodyLimitLayer::new(65556)) // 64 MiB request payload limit
      .layer(SetResponseHeaderLayer::if_not_present(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, max-age=0"),
      ))
      .layer(TimeoutLayer::new(Duration::from_secs(30)))
      .layer(CompressionLayer::new().br(true).gzip(true).deflate(true));

    let probes = Router::new()
      .route("/health", get(probe::get_health))
      .route("/readiness", get(probe::get_readiness))
      .fallback(|| async { StatusCode::NOT_FOUND })
      .layer(CorsLayer::permissive());

    let router = Router::new()
      .nest("/api", data_api)
      .nest("/probes", probes)
      .merge(hypermedia_api)
      .layer(middleware)
      .with_state(router_state);

    let router = if shared_config.http_server.base_path.is_empty() {
      router.into_service()
    } else {
      Router::new()
        .nest(shared_config.http_server.base_path.as_str(), router)
        .into_service()
    };

    let app = NormalizePathLayer::trim_trailing_slash().layer(router);

    axum::serve(listener, app.into_make_service())
      .with_graceful_shutdown(close_signal)
      .await
  }
}

#[inline]
fn create_data_routes(
  auth_user_store: &Option<Arc<UserStore>>,
  server_key: Arc<[u8]>,
  server_state: Arc<RwLock<ServerState>>,
) -> Router<RouterState> {
  let data_api = Router::new()
    .route("/ups", get(json_api::route::ups_list::get))
    .route("/ups/{ups_name}", get(json_api::route::ups::get))
    .route(
      "/ups/{ups_name}",
      patch(json_api::route::rw::patch).route_layer(
        ServiceBuilder::new().option_layer(
          auth_user_store
            .as_ref()
            .map(|_| AuthorizeApiLayer::new(Permissions::SET_VAR)),
        ),
      ),
    )
    .route(
      "/ups/{ups_name}/instcmd",
      post(json_api::route::instcmd::post).route_layer(
        ServiceBuilder::new().option_layer(
          auth_user_store
            .as_ref()
            .map(|_| AuthorizeApiLayer::new(Permissions::INSTCMD)),
        ),
      ),
    )
    .route(
      "/ups/{ups_name}/fsd",
      post(json_api::route::fsd::post).route_layer(
        ServiceBuilder::new()
          .option_layer(
            auth_user_store
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
          auth_user_store
            .as_ref()
            .map(|_| ApiAuthLayer::new(server_key)),
        )
        .layer(DaemonStateLayer::new(server_state)),
    );

  data_api
}

#[inline]
fn create_hypermedia_routes(
  auth_user_store: &Option<Arc<UserStore>>,
  server_key: Arc<[u8]>,
  config: Arc<ServerConfig>,
) -> Router<RouterState> {
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
    .route(
      "/ups/{ups_name}/instcmd",
      post(hypermedia::route::ups::instcmd::post).route_layer(
        ServiceBuilder::new().option_layer(
          auth_user_store
            .as_ref()
            .map(|_| AuthorizeUserLayer::new(config.clone(), Permissions::INSTCMD)),
        ),
      ),
    )
    .route(
      "/ups/{ups_name}/fsd",
      post(hypermedia::route::ups::fsd::post).route_layer(
        ServiceBuilder::new().option_layer(
          auth_user_store
            .as_ref()
            .map(|_| AuthorizeUserLayer::new(config.clone(), Permissions::FSD)),
        ),
      ),
    )
    .route(
      "/ups/{ups_name}/rw",
      patch(hypermedia::route::ups::rw::patch).route_layer(
        ServiceBuilder::new().option_layer(
          auth_user_store
            .as_ref()
            .map(|_| AuthorizeUserLayer::new(config.clone(), Permissions::SET_VAR)),
        ),
      ),
    )
    .route("/", get(hypermedia::route::home::get))
    .route("/server", get(hypermedia::route::server_info::get))
    .route("/ups/{ups_name}", get(hypermedia::route::ups::get));

  match auth_user_store {
    Some(store) => hypermedia_api
      .layer(RenewSessionLayer::new(
        server_key.clone(),
        store.clone(),
        AUTH_COOKIE_RENEW,
      ))
      .route("/logout", post(hypermedia::route::logout::post))
      .layer(UserAuthLayer::new(
        server_key,
        store.clone(),
        format!("{}/login", config.http_server.base_path),
      ))
      .route(
        "/login",
        get(hypermedia::route::login::get).post(hypermedia::route::login::post),
      ),
    _ => hypermedia_api,
  }
  .route("/not-found", get(hypermedia::route::not_found::get))
  .route(
    "/_layout/themes",
    get(hypermedia::route::layout::get_themes),
  )
  .nest_service("/static", static_files)
  .fallback(hypermedia::route::not_found::get)
}
