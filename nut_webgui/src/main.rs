use self::config::{
  ServerConfig, cfg_args::ServerCliArgs, cfg_env::ServerEnvArgs, cfg_toml::ServerTomlArgs,
};
use crate::{config::error::ConfigError, skip_tls_verifier::SkipTlsVerifier};
use event::EventChannel;
use http::HttpServer;
use nut_webgui_upsmc::{
  clients::{NutPoolClient, NutPoolClientBuilder},
  rustls::{ClientConfig, pki_types::ServerName},
};
use rustls_platform_verifier::BuilderVerifierExt;
use service::{
  BackgroundServiceRunner, sync_desc::DescriptionSyncService, sync_device::DeviceSyncService,
  sync_status::StatusSyncService,
};
use state::{DaemonState, ServerState};
use std::{collections::HashMap, panic, sync::Arc, time::Duration};
use tokio::{
  net::TcpListener,
  select,
  signal::{self, unix::SignalKind},
  sync::RwLock,
};
use tracing::{debug, error, info, warn};

mod config;
mod device_entry;
mod diff_utils;
mod event;
mod http;
mod service;
mod skip_tls_verifier;
mod state;

#[inline]
fn load_configs() -> Result<ServerConfig, ConfigError> {
  let cli_args = ServerCliArgs::load()?;

  let env_args = if cli_args.allow_env {
    ServerEnvArgs::load()?
  } else {
    ServerEnvArgs::default()
  };

  let toml_path = cli_args
    .config_file
    .as_ref()
    .or(env_args.config_file.as_ref());

  let toml_args = if let Some(path) = toml_path {
    ServerTomlArgs::load(path)?
  } else {
    ServerTomlArgs::default()
  };

  let config = ServerConfig::new()
    .layer(toml_args)
    .layer(env_args)
    .layer(cli_args);

  Ok(config)
}

#[inline]
fn create_pool(
  config: &ServerConfig,
) -> Result<NutPoolClient, Box<dyn core::error::Error + 'static>> {
  let tls_client_conf = match config.upsd.tls_mode {
    config::tls_mode::TlsMode::Disable => None,
    config::tls_mode::TlsMode::Strict => Some(
      ClientConfig::builder()
        .with_platform_verifier()
        .with_no_client_auth(),
    ),
    config::tls_mode::TlsMode::SkipVerify => {
      let mut config = ClientConfig::builder()
        .with_platform_verifier()
        .with_no_client_auth();

      config
        .dangerous()
        .set_certificate_verifier(Arc::new(SkipTlsVerifier));

      Some(config)
    }
  };

  let mut builder = NutPoolClientBuilder::new(config.upsd.get_socket_addr().into())
    .with_timeout(Duration::from_secs(config.upsd.poll_freq))
    .with_limit(config.upsd.max_conn);

  if let Some(tls_config) = tls_client_conf {
    let server_name = ServerName::try_from(config.upsd.addr.as_ref().to_owned())?;
    builder = builder.with_tls(server_name, Arc::new(tls_config));
  }

  Ok(builder.build())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn core::error::Error>> {
  let mut sigterm = signal::unix::signal(SignalKind::terminate()).expect("SIGTERM stream failed");
  let mut sigint = signal::unix::signal(SignalKind::interrupt()).expect("SIGINT stream failed");
  let mut sigquit = signal::unix::signal(SignalKind::quit()).expect("SIGQUIT stream failed");

  panic::set_hook(Box::new(|info| {
    eprintln!(
      "Application thread panicked. Aborting process. details={}",
      info
    );

    std::process::exit(-1);
  }));

  let config = match load_configs() {
    Ok(cfg) => cfg,
    Err(ConfigError::File(err)) => {
      eprintln!("invalid file config, reason = {err}");
      std::process::exit(2);
    }
    Err(ConfigError::Environment(err)) => {
      eprintln!("invalid env config, reason = {err}");
      std::process::exit(2);
    }
    Err(ConfigError::Arguments(err)) => {
      err.print()?;
      err.exit();
    }
  };

  tracing_subscriber::fmt()
    .with_max_level(config.log_level)
    .init();

  debug!(message = "server initialized", config = ?config);

  let listener = TcpListener::bind(config.http_server.get_listen_addr())
    .await
    .inspect_err(|err| error!(message = "cannot bind tcp socket to listen", reason = %err, listen_port = config.http_server.port))?;

  let client_pool = create_pool(&config)?;
  let event_channel = EventChannel::new(64);
  let server_state = Arc::new(RwLock::new(ServerState {
    remote_state: DaemonState::new(),
    devices: HashMap::new(),
    shared_desc: HashMap::new(),
  }));

  let device_sync = DeviceSyncService::new(
    client_pool.clone(),
    event_channel.clone(),
    server_state.clone(),
    Duration::from_secs(config.upsd.poll_freq),
  );

  let desc_sync = DescriptionSyncService::new(
    client_pool.clone(),
    event_channel.clone(),
    server_state.clone(),
  );

  let status_sync = StatusSyncService::new(
    client_pool.clone(),
    event_channel.clone(),
    server_state.clone(),
    Duration::from_secs(config.upsd.poll_interval),
    Duration::from_secs(config.upsd.poll_freq),
  );

  let bg_services = BackgroundServiceRunner::new()
    .with_max_timeout(Duration::from_secs(10))
    .add_service(device_sync)
    .add_service(desc_sync)
    .add_service(status_sync)
    .start();

  let pool_handle = client_pool.clone();
  let close_signal = async move {
    select! {
    _ = sigterm.recv() => { info!("SIGTERM signal received."); }
    _ = sigquit.recv() => { info!("SIGQUIT signal received."); }
    _ = sigint.recv() => { info!("SIGINT signal received."); }
    };

    info!("shutting down background services");

    if let Err(err) = bg_services.stop().await {
      warn!(message = "some services are not shutdown properly", reason=%err)
    }

    info!("closing open upsd connections");
    _ = pool_handle.close().await;
  };

  HttpServer::new(config, server_state, client_pool)
    .serve(listener, close_signal)
    .await
    .inspect_err(|err| {
      error!(message = "http server failed", reason = %err);
    })?;

  info!(message = "http server is closed");

  Ok(())
}
