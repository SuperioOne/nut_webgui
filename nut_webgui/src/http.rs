mod hypermedia;
mod json;
mod middlewares;
mod probes;
mod problem_detail;

use crate::{config::ServerConfig, state::ServerState};
use axum::{
  Router, ServiceExt,
  http::{HeaderValue, StatusCode, header},
  routing::{get, patch, post},
};
use hypermedia::routes;
use middlewares::{
  daemon_status::DaemonStateLayer, validate_content_length::ValidateEmptyContentLength,
};
use problem_detail::ProblemDetail;
use std::{sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::RwLock};
use tower::{Layer, ServiceBuilder};
use tower_http::{
  compression::CompressionLayer, cors::CorsLayer, limit::RequestBodyLimitLayer,
  normalize_path::NormalizePathLayer, set_header::SetResponseHeaderLayer, timeout::TimeoutLayer,
  trace::TraceLayer, validate_request::ValidateRequestHeaderLayer,
};

#[derive(Clone, Debug)]
struct RouterState {
  config: Arc<ServerConfig>,
  state: Arc<RwLock<ServerState>>,
}

pub struct HttpServer {
  config: ServerConfig,
  server_state: Arc<RwLock<ServerState>>,
}

impl HttpServer {
  pub fn new(config: ServerConfig, server_state: Arc<RwLock<ServerState>>) -> Self {
    Self {
      config,
      server_state,
    }
  }

  pub async fn serve<F>(self, listener: TcpListener, close_signal: F) -> Result<(), std::io::Error>
  where
    F: Future<Output = ()> + Send + 'static,
  {
    let Self {
      server_state,
      config,
    } = self;

    let middleware = ServiceBuilder::new()
      .layer(CompressionLayer::new().br(true).gzip(true).deflate(true))
      .layer(RequestBodyLimitLayer::new(65556)) // 64 MiB request payload limit
      .layer(TraceLayer::new_for_http())
      .layer(TimeoutLayer::new(Duration::from_secs(30)))
      .layer(SetResponseHeaderLayer::if_not_present(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, max-age=0"),
      ));

    let probes = Router::new()
      .route("/health", get(probes::get_health))
      .route("/readiness", get(probes::get_readiness))
      .fallback(|| async { StatusCode::NOT_FOUND })
      .layer(CorsLayer::permissive());

    let data_api = Router::new()
      .route("/ups", get(json::get_ups_list))
      .route("/ups/{ups_name}", get(json::get_ups_by_name))
      .route("/ups/{ups_name}", patch(json::patch_var))
      .route(
        "/ups/{ups_name}/instcmd",
        post(json::post_command),
      )
      .route(
        "/ups/{ups_name}/fsd",
        post(json::post_fsd).layer(ValidateRequestHeaderLayer::custom(
          ValidateEmptyContentLength,
        )),
      )
      .fallback(|| async { ProblemDetail::new("Target resource not found", StatusCode::NOT_FOUND) })
      .layer(DaemonStateLayer::new(server_state.clone()))
      .layer(ValidateRequestHeaderLayer::accept("application/json"))
      .layer(CorsLayer::permissive());

    let static_files = Router::new()
      .route("/style.css", get(routes::static_content::get_css))
      .route("/index.js", get(routes::static_content::get_javascript))
      .route("/icon.svg", get(routes::static_content::get_icon))
      .route(
        "/feather-sprite.svg",
        get(routes::static_content::get_sprite_sheet),
      )
      .fallback(|| async { StatusCode::NOT_FOUND });

    let hypermedia_api = Router::new()
      .nest_service("/static", static_files)
      .route(
        "/_layout/themes",
        get(hypermedia::routes::layout::get_themes),
      )
      .route(
        "/ups/{ups_name}/instcmd",
        post(hypermedia::routes::ups::post_instcmd),
      )
      .route(
        "/ups/{ups_name}/fsd",
        post(hypermedia::routes::ups::post_fsd),
      )
      .route(
        "/ups/{ups_name}/rw",
        patch(hypermedia::routes::ups::patch_rw),
      )
      .route("/", get(hypermedia::routes::home::get))
      .route("/not-found", get(hypermedia::routes::not_found::get))
      .route("/server", get(hypermedia::routes::server_info::get))
      .route("/ups/{ups_name}", get(hypermedia::routes::ups::get))
      .fallback(hypermedia::routes::not_found::get);

    let shared_config = Arc::new(config);
    let router_state = RouterState {
      config: shared_config.clone(),
      state: server_state,
    };

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
