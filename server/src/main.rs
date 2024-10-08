mod http_server;
pub mod ups_daemon_state;
mod ups_services;
mod upsd_client;

use crate::{
  http_server::{start_http_server, HttpServerConfig, UpsdConfig},
  ups_services::{
    upsd_poll_service, upsd_state_service, UpsPollerConfig, UpsStorageConfig, UpsUpdateMessage,
  },
};
use clap::Parser;
use std::{
  net::{IpAddr, Ipv4Addr, SocketAddr},
  panic,
  sync::Arc,
};
use tokio::{
  select,
  signal::{self, unix::SignalKind},
  sync::{
    mpsc::{self, Receiver, Sender},
    RwLock,
  },
  time::Duration,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use tracing_subscriber::fmt;
use ups_daemon_state::UpsDaemonState;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct ServerArgs {
  /// Non-critical ups variables update frequency in seconds.
  #[arg(long, default_value_t = 30)]
  poll_freq: u64,

  /// Critical ups variables update interval in seconds.
  #[arg(long, default_value_t = 2)]
  poll_interval: u64,

  /// NUT server address
  #[arg(long, default_value_t = String::from("localhost"))]
  upsd_addr: String,

  /// NUT server port
  #[arg(long, default_value_t = 3493)]
  upsd_port: u16,

  /// NUT username
  #[arg(long)]
  upsd_user: Option<String>,

  /// NUT password
  #[arg(long)]
  upsd_pass: Option<String>,

  /// Listen address for HTTP server
  #[arg(short, long, default_value_t = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))]
  listen: IpAddr,

  /// HTTP server port
  #[arg(short, long, default_value_t = 9000)]
  port: u16,

  /// Log level
  #[arg(long, default_value_t = tracing::Level::INFO)]
  log_level: tracing::Level,

  /// Static file directory
  #[arg(long, default_value_t = String::from("static"))]
  static_dir: String,
}

#[tokio::main]
async fn main() {
  let args = ServerArgs::parse();

  fmt().with_max_level(args.log_level).init();
  debug!(
    message = "Server initialized with",
    poll_interval = &args.poll_interval,
    poll_freq = &args.poll_freq,
    upsd_addr = &args.upsd_addr,
    upsd_port = &args.upsd_port,
    upsd_user = &args.upsd_user,
    upsd_pass = args.upsd_pass.as_deref().map(|_| "[REDACTED]"),
    listen = &args.listen.to_string(),
    port = &args.port,
    log_level = args.log_level.as_str(),
    static_dir = &args.static_dir
  );

  let cancellation = CancellationToken::new();
  let (tx, rx): (Sender<UpsUpdateMessage>, Receiver<UpsUpdateMessage>) = mpsc::channel(256);
  let state_arc = Arc::new(RwLock::new(UpsDaemonState::new()));
  let upsd_address = format!("{}:{}", args.upsd_addr, args.upsd_port);
  let (poll_interval, poll_freq) = {
    if args.poll_freq < args.poll_interval {
      warn!("Poll interval is set greater than or equal to poll frequency.");
      (args.poll_interval, args.poll_interval)
    } else {
      (args.poll_interval, args.poll_freq)
    }
  };

  panic::set_hook(Box::new(|info| {
    error!("Panic details: {}", info);
    std::process::abort();
  }));

  // Spawns background services
  let poll_srv_handle = upsd_poll_service(UpsPollerConfig {
    address: upsd_address.clone(),
    poll_freq: Duration::from_secs(poll_freq),
    poll_interval: Duration::from_secs(poll_interval),
    write_channel: tx,
    cancellation: cancellation.clone(),
  });

  let state_srv_handle = upsd_state_service(UpsStorageConfig {
    read_channel: rx,
    cancellation: cancellation.clone(),
    upsd_state: state_arc.clone(),
  });

  // http server
  let server_handle = start_http_server(HttpServerConfig {
    upsd_state: state_arc,
    listen: SocketAddr::new(args.listen, args.port),
    static_dir: args.static_dir,
    upsd_config: UpsdConfig {
      addr: upsd_address,
      pass: args.upsd_pass,
      user: args.upsd_user,
      poll_freq: Duration::from_secs(poll_freq),
      poll_interval: Duration::from_secs(poll_interval),
    },
  });

  let mut sigterm = signal::unix::signal(SignalKind::terminate()).expect("SIGTERM stream failed");
  let mut sigint = signal::unix::signal(SignalKind::interrupt()).expect("SIGINT stream failed");
  let mut sigquit = signal::unix::signal(SignalKind::interrupt()).expect("SIGQUIT stream failed");

  select! {
    _ = sigterm.recv() => { info!("SIGTERM signal received.");}
    _ = sigquit.recv() => { info!("SIGQUIT signal received.");}
    _ = sigint.recv() => { info!("SIGINT signal received.");}
  }

  cancellation.cancel();

  info!("Shutting down services");
  _ = poll_srv_handle.await;
  _ = state_srv_handle.await;
  info!("Shutting http server");
  server_handle.abort();
}
