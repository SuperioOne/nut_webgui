use crate::ups_daemon_state::UpsDaemonState;
use axum::{
  http::{header::CACHE_CONTROL, HeaderValue, StatusCode},
  routing::{get, post},
  Router, ServiceExt,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{spawn, sync::RwLock, task::JoinHandle};
use tower::{Layer, ServiceBuilder};
use tower_http::{
  compression::CompressionLayer, cors::CorsLayer, normalize_path::NormalizePathLayer,
  services::ServeDir, set_header::SetResponseHeaderLayer, timeout::TimeoutLayer, trace::TraceLayer,
};

use self::middlewares::DaemonStateLayer;

mod common;
mod hypermedia;
mod json;
mod middlewares;
mod probes;

pub struct HttpServerConfig {
  pub listen: SocketAddr,
  pub upsd_state: Arc<RwLock<UpsDaemonState>>,
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

#[derive(Clone)]
pub(crate) struct ServerState {
  pub upsd_state: Arc<RwLock<UpsDaemonState>>,
  pub upsd_config: Arc<UpsdConfig>,
}

pub fn start_http_server(config: HttpServerConfig) -> JoinHandle<()> {
  spawn(async move {
    let HttpServerConfig {
      upsd_state,
      listen,
      upsd_config,
      static_dir,
    } = config;

    let middleware = ServiceBuilder::new()
      .layer(CompressionLayer::new().br(true).gzip(true).deflate(true))
      .layer(TraceLayer::new_for_http())
      .layer(TimeoutLayer::new(Duration::from_secs(10)))
      .layer(SetResponseHeaderLayer::if_not_present(
        CACHE_CONTROL,
        HeaderValue::from_static("no-cache, max-age=0"),
      ));

    let probes = Router::new()
      .route("/health", get(probes::get_health))
      .route("/readiness", get(probes::get_readiness))
      .fallback(|| async { StatusCode::NOT_FOUND })
      .layer(CorsLayer::permissive());

    let data_api = Router::new()
      .route("/ups/:ups_name", get(json::get_ups_by_name))
      .route("/ups", get(json::get_ups_list))
      .route("/ups/:ups_name/command", post(json::post_command))
      .fallback(|| async { StatusCode::NOT_FOUND })
      .layer(DaemonStateLayer::new(upsd_state.clone()))
      .layer(CorsLayer::permissive());

    let hypermedia_api = Router::new()
      .route("/ups/:ups_name", get(hypermedia::routes::ups::get))
      .route(
        "/ups/:ups_name/command",
        post(hypermedia::routes::ups::post_command),
      )
      .route("/", get(hypermedia::routes::home::get))
      .route("/not-found", get(hypermedia::routes::not_found::get))
      .fallback(hypermedia::routes::not_found::get);

    let state = ServerState {
      upsd_state,
      upsd_config: Arc::new(upsd_config),
    };

    let router = Router::new()
      .nest_service("/static", ServeDir::new(static_dir))
      .nest("/api", data_api)
      .nest("/probes", probes)
      .merge(hypermedia_api)
      .layer(middleware)
      .with_state(state)
      .into_service();

    let app = NormalizePathLayer::trim_trailing_slash().layer(router);
    let listener = tokio::net::TcpListener::bind(listen).await.unwrap();

    axum::serve(listener, app.into_make_service())
      .await
      .unwrap();
  })
}
