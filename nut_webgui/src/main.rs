mod config;
mod device_entry;
mod diff_utils;
mod event;
mod http;
mod service;
mod state;
mod uri_path;

use self::config::{
  ServerConfig, cfg_args::ServerCliArgs, cfg_env::ServerEnvArgs, cfg_toml::ServerTomlArgs,
};
use crate::config::error::ConfigError;
use event::EventChannel;
use http::HttpServer;
use nut_webgui_upsmc::clients::{NutPoolClient, NutPoolClientBuilder};
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

  let client_pool = NutPoolClientBuilder::new(config.upsd.get_socket_addr())
    .with_timeout(Duration::from_secs(config.upsd.poll_freq))
    .with_limit(config.upsd.max_conn)
    .build();

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
    _ = client_pool.close().await;
  };

  HttpServer::new(config, server_state)
    .serve(listener, close_signal)
    .await
    .inspect_err(|err| {
      error!(message = "http server failed", reason = %err);
    })?;

  info!(message = "http server is closed");

  Ok(())
}
