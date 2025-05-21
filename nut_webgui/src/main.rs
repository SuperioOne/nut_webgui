// mod http;
mod config;
mod event;
mod state;
mod sync;
mod uri_path;

use event::EventChannel;
use nut_webgui_upsmc::clients::NutPoolClient;
// use crate::{
//   http::{HttpServerConfig, UpsdConfig, start_http_server},
//   ups_services::{
//     UpsPollerConfig, UpsStorageConfig, UpsUpdateMessage, upsd_poll_service, upsd_state_service,
//   },
// };
use self::config::{
  ServerConfig, cfg_args::ServerCliArgs, cfg_env::ServerEnvArgs, cfg_toml::ServerTomlArgs,
};
use state::{DaemonState, ServerState};
use std::{collections::HashMap, panic, sync::Arc, time::Duration};
use sync::{desc_sync::DescriptionSyncService, ups_sync::DeviceSyncService};
use tokio::{
  select,
  signal::{self, unix::SignalKind},
  sync::RwLock,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

macro_rules! timeout {
  ($handle:expr, seconds = $time:expr, message = $message:literal) => {
    if let Err(_) = tokio::time::timeout(std::time::Duration::from_secs($time), $handle).await {
      tracing::warn!(message = $message);
    }
  };
}

fn load_configs() -> ServerConfig {
  let cli_args = ServerCliArgs::load();

  let env_args = if cli_args.allow_env {
    ServerEnvArgs::load().unwrap()
  } else {
    ServerEnvArgs::default()
  };

  let toml_path = cli_args
    .config_file
    .as_ref()
    .or_else(|| env_args.config_file.as_ref());

  let toml_args = if let Some(path) = toml_path {
    ServerTomlArgs::load(path).unwrap()
  } else {
    ServerTomlArgs::default()
  };

  let config = ServerConfig::new()
    .layer(toml_args)
    .layer(env_args)
    .layer(cli_args);

  config
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  panic::set_hook(Box::new(|info| {
    eprintln!(
      "Application thread panicked. Aborting process. details={}",
      info
    );

    std::process::exit(-1);
  }));

  let configs = Arc::new(load_configs());

  tracing_subscriber::fmt()
    .with_max_level(configs.log_level)
    .init();

  debug!(message = "Server initialized.", config = ?configs);

  let socket_addr = format!(
    "{addr}:{port}",
    addr = &configs.upsd.addr,
    port = &configs.upsd.port
  );

  let event_channel = EventChannel::new(50);
  let client_pool = NutPoolClient::new(socket_addr, configs.upsd.max_conn);
  let cancellation = CancellationToken::new();
  let server_state = Arc::new(RwLock::new(ServerState {
    state: DaemonState::new(),
    devices: HashMap::new(),
    shared_desc: HashMap::new(),
  }));

  let desc_service_handle = tokio::spawn(
    DescriptionSyncService::new(
      client_pool.clone(),
      event_channel.clone(),
      server_state.clone(),
      cancellation.clone(),
    )
    .run(),
  );

  let sync_service_handle = tokio::spawn(
    DeviceSyncService::new(
      client_pool.clone(),
      event_channel.clone(),
      server_state.clone(),
      Duration::from_secs(configs.upsd.poll_freq),
      cancellation.clone(),
    )
    .run(),
  );

  // // http server
  // let server_handle = start_http_server(HttpServerConfig {
  //   upsd_state: state_arc,
  //   listen: SocketAddr::new(cli_args.listen, cli_args.port),
  //   static_dir: cli_args.static_dir,
  //   upsd_config: UpsdConfig {
  //     addr: upsd_address,
  //     pass: cli_args.upsd_pass,
  //     user: cli_args.upsd_user,
  //     poll_freq: Duration::from_secs(poll_freq),
  //     poll_interval: Duration::from_secs(poll_interval),
  //   },
  // });
  //
  let mut sigterm = signal::unix::signal(SignalKind::terminate()).expect("SIGTERM stream failed");
  let mut sigint = signal::unix::signal(SignalKind::interrupt()).expect("SIGINT stream failed");
  let mut sigquit = signal::unix::signal(SignalKind::interrupt()).expect("SIGQUIT stream failed");

  select! {
    _ = sigterm.recv() => { info!("SIGTERM signal received.");}
    _ = sigquit.recv() => { info!("SIGQUIT signal received.");}
    _ = sigint.recv() => { info!("SIGINT signal received.");}
  }

  cancellation.cancel();

  info!("shutting down services");

  timeout!(
    sync_service_handle,
    seconds = 5,
    message = "sync service shutdown took too long, aborting service forcefully"
  );

  timeout!(
    desc_service_handle,
    seconds = 5,
    message = "description service shutdown took too long, aborting service forcefully"
  );

  _ = client_pool.close().await;

  Ok(())
}
