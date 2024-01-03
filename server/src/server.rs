use std::net::{SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use axum::Router;
use axum::routing::{get, post};
use tokio::spawn;
use tokio::sync::{RwLock};
use tokio::task::JoinHandle;
use tower::ServiceBuilder;
use tower_http::timeout::TimeoutLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use crate::ups_mem_store::UpsStore;

pub mod utils;
pub mod ups_info;
pub mod notifications;
mod routes;

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
      static_dir
    } = config;

    let state = Arc::new(ServerState {
      store,
      upsd_config,
    });

    let middleware = ServiceBuilder::new()
      .layer(TraceLayer::new_for_http())
      .layer(TimeoutLayer::new(Duration::from_secs(30)));

    let app = Router::new()
      .nest_service("/static", ServeDir::new(static_dir))
      .route("/ups/:ups_name", get(routes::ups::handler))
      .route("/ups/:ups_name/command", post(routes::ups::handler_command))
      .route("/", get(routes::home::handler))
      .route("/not-found", get(routes::not_found::handler))
      .fallback(routes::not_found::handler)
      .layer(middleware)
      .with_state(state);

    let listener = tokio::net::TcpListener::bind(listen).await.unwrap();
    axum::serve(listener, app).await.unwrap();
  })
}
