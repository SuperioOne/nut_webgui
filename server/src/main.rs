mod ups_service;
mod server;
mod upsd_client;
pub mod ups_mem_store;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::panic;
use std::sync::Arc;
use clap::Parser;
use tokio::{select, signal};
use tokio::signal::unix::SignalKind;
use tokio::sync::{mpsc, RwLock};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_util::sync::CancellationToken;
use tokio::time::Duration;
use tracing::{debug, error, info};
use tracing_subscriber::fmt;
use ups_mem_store::UpsStore;
use crate::server::{HttpServerConfig, start_http_server, UpsdConfig};
use crate::ups_service::{UpsUpdateMessage};
use crate::ups_service::storage_service::{ups_storage_service, UpsStorageConfig};
use crate::ups_service::ups_poll_service::{ups_poller_service, UpsPollerConfig};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct ServerArgs {
  /// UPS info update frequency in seconds.
  #[arg(long, default_value_t = 10)]
  poll_freq: u64,

  /// NUT server address
  #[arg(long, default_value_t = String::from("localhost"))]
  upsd_addr: String,

  /// NUT server address
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

  fmt()
    .with_max_level(args.log_level)
    .init();
  debug!("Server initialized with {:?}", &args);

  let cancellation = CancellationToken::new();
  let (tx, rx): (Sender<UpsUpdateMessage>, Receiver<UpsUpdateMessage>) = mpsc::channel(4096);
  let store = UpsStore::new("default");
  let store_arc = Arc::new(RwLock::new(store));

  panic::set_hook(Box::new(|info| {
    error!("Panic details: {}", info);
    std::process::abort();
  }));

  // spawn background services
  let upsd_address = format!("{}:{}", args.upsd_addr, args.upsd_port);
  let poll_service_handle = ups_poller_service(UpsPollerConfig {
    address: upsd_address.clone(),
    poll_freq: Duration::from_secs(args.poll_freq),
    write_channel: tx.clone(),
    cancellation: cancellation.clone(),
  });
  let store_service_handle = ups_storage_service(UpsStorageConfig {
    read_channel: rx,
    cancellation: cancellation.clone(),
    store: store_arc.clone(),
  });

  // http server
  let server_handle = start_http_server(HttpServerConfig {
    store: store_arc.clone(),
    listen: SocketAddr::new(args.listen, args.port),
    static_dir: args.static_dir,
    upsd_config: UpsdConfig {
      addr: upsd_address,
      pass: args.upsd_pass,
      user: args.upsd_user,
    },
  });

  let mut sigterm = signal::unix::signal(SignalKind::terminate()).expect("SIGTERM stream failed");
  let mut sigint = signal::unix::signal(SignalKind::interrupt()).expect("SIGINT stream failed");
  let mut sigquit = signal::unix::signal(SignalKind::interrupt()).expect("SIGQUIT stream failed");

  select! {
        _ = sigterm.recv() => {
           cancellation.cancel();
            drop(tx);

            info!("Shutting down services");
            _ = poll_service_handle.await;
            _ = store_service_handle.await;
            info!("Shutting http server");
            server_handle.abort();
        }
        _ = sigquit.recv() => {
           cancellation.cancel();
            drop(tx);

            info!("Shutting down services");
            _ = poll_service_handle.await;
            _ = store_service_handle.await;
            info!("Shutting http server");
            server_handle.abort();
        }
        _ = sigint.recv() => {
            cancellation.cancel();
            drop(tx);

            info!("Shutting down services");
            _ = poll_service_handle.await;
            _ = store_service_handle.await;
            info!("Shutting http server");
            server_handle.abort();
        }
      };
}
