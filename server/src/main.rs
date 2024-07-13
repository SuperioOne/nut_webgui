mod http_server;
pub mod ups_mem_store;
mod ups_service;
mod upsd_client;

use crate::{
  http_server::{start_http_server, HttpServerConfig, UpsdConfig},
  ups_service::{
    storage_service::{ups_storage_service, UpsStorageConfig},
    ups_poll_service::{ups_polling_service, UpsPollerConfig},
    UpsUpdateMessage,
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
use ups_mem_store::UpsStore;

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
  debug!("Server initialized with {:?}", &args);

  let cancellation = CancellationToken::new();
  let (tx, rx): (Sender<UpsUpdateMessage>, Receiver<UpsUpdateMessage>) = mpsc::channel(4096);
  let store = UpsStore::new();
  let store_arc = Arc::new(RwLock::new(store));
  let upsd_address = format!("{}:{}", args.upsd_addr, args.upsd_port);
  let (poll_interval, poll_freq) = if args.poll_freq < args.poll_interval {
    warn!("Poll interval is set greater than or equal to poll frequency. Update scheduler will only use full update");
    (args.poll_interval, args.poll_interval)
  } else {
    (args.poll_interval, args.poll_freq)
  };

  panic::set_hook(Box::new(|info| {
    error!("Panic details: {}", info);
    std::process::abort();
  }));

  // Spawns background services
  let poll_service_handle = ups_polling_service(UpsPollerConfig {
    address: upsd_address.clone(),
    poll_freq: Duration::from_secs(poll_freq),
    poll_interval: Duration::from_secs(poll_interval),
    write_channel: tx,
    cancellation: cancellation.clone(),
  });

  let store_service_handle = ups_storage_service(UpsStorageConfig {
    read_channel: rx,
    cancellation: cancellation.clone(),
    store: store_arc.clone(),
  });

  // http server
  let server_handle = start_http_server(HttpServerConfig {
    store: store_arc,
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
  _ = poll_service_handle.await;
  _ = store_service_handle.await;
  info!("Shutting http server");
  server_handle.abort();
}
