use crate::ups_mem_store::UpsStore;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::spawn;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

mod hypermedia;
mod json;

pub struct HttpServerConfig {
  pub listen: SocketAddr,
  pub store: Arc<RwLock<UpsStore>>,
  pub upsd_config: UpsdConfig,
  pub static_dir: String,
}

pub(crate) struct UpsdConfig {
  pub pass: Option<String>,
  pub user: Option<String>,
  pub addr: String,
  pub poll_freq: Duration,
  pub poll_interval: Duration,
}

pub(crate) struct ServerState {
  pub store: Arc<RwLock<UpsStore>>,
  pub upsd_config: UpsdConfig,
}

pub fn start_http_server(config: HttpServerConfig) -> JoinHandle<()> {
  spawn(async move {
    let HttpServerConfig {
      store,
      listen,
      upsd_config,
      static_dir,
    } = config;

    let state = Arc::new(ServerState { store, upsd_config });

    let middleware = ServiceBuilder::new()
      .layer(CompressionLayer::new().br(true).gzip(true).deflate(true))
      .layer(TraceLayer::new_for_http())
      .layer(TimeoutLayer::new(Duration::from_secs(10)));

    let probes = Router::new()
      .route("/health", get(|| async { StatusCode::OK }))
      .fallback(|| async { StatusCode::NOT_FOUND });

    let data_api = Router::new()
      .route("/ups/:ups_name", get(json::get_ups_by_name))
      .route("/ups", get(json::get_ups_list))
      .route("/ups/:ups_name/command", post(json::post_command))
      .fallback(|| async { StatusCode::NOT_FOUND })
      .layer(CorsLayer::permissive());

    let hypermedia_api = Router::new()
      .nest_service("/static", ServeDir::new(static_dir))
      .route("/ups/:ups_name", get(hypermedia::routes::ups::get))
      .route(
        "/ups/:ups_name/command",
        post(hypermedia::routes::ups::post_command),
      )
      .route("/", get(hypermedia::routes::home::get))
      .route("/not-found", get(hypermedia::routes::not_found::get))
      .fallback(hypermedia::routes::not_found::get);

    let app = Router::new()
      .nest("/api", data_api)
      .nest("/probes", probes)
      .merge(hypermedia_api)
      .layer(middleware)
      .with_state(state);

    let listener = tokio::net::TcpListener::bind(listen).await.unwrap();
    axum::serve(listener, app).await.unwrap();
  })
}
