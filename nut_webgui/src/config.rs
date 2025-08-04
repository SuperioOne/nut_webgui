use crate::config::{tls_mode::TlsMode, uri_path::UriPath};
use core::net::{IpAddr, Ipv4Addr};
use std::{num::NonZeroUsize, path::PathBuf};
use tracing::Level;

mod macros;

pub mod cfg_args;
pub mod cfg_env;
pub mod cfg_toml;
pub mod error;
pub mod tls_mode;
pub mod uri_path;

pub trait ConfigLayer {
  fn apply_layer(self, config: ServerConfig) -> ServerConfig;
}

#[derive(Debug)]
pub struct ServerConfig {
  pub config_file: Option<PathBuf>,
  pub default_theme: Option<Box<str>>,
  pub log_level: tracing::Level,

  pub http_server: HttpServerConfig,
  pub upsd: UpsdConfig,
}

#[derive(Debug)]
pub struct HttpServerConfig {
  pub listen: IpAddr,
  pub port: u16,
  pub base_path: UriPath,
}

pub struct UpsdConfig {
  /// Poll frequency in seconds for less critical parameters
  pub poll_freq: u64,

  /// Poll interval in seconds for ups status
  pub poll_interval: u64,

  /// UPSD TCP address
  /// It can be hostname, IPv4, or IPv6
  pub addr: Box<str>,

  /// UPSD TCP port
  pub port: u16,

  /// UPSD username
  pub user: Option<Box<str>>,

  /// UPSD password
  pub pass: Option<Box<str>>,

  /// Maximum allowed connection limit aka pool size
  pub max_conn: NonZeroUsize,

  /// UPSD starts with TLS
  pub tls_mode: TlsMode,
}

impl UpsdConfig {
  pub fn get_socket_addr(&self) -> String {
    format!("{address}:{port}", address = self.addr, port = self.port)
  }
}

impl HttpServerConfig {
  pub fn get_listen_addr(&self) -> String {
    format!("{ip}:{port}", ip = self.listen, port = self.port)
  }
}

impl Default for HttpServerConfig {
  fn default() -> Self {
    Self {
      listen: Ipv4Addr::UNSPECIFIED.into(),
      base_path: UriPath::default(),
      port: 9000,
    }
  }
}

impl Default for UpsdConfig {
  fn default() -> Self {
    Self {
      pass: None,
      user: None,
      addr: "127.0.0.1".into(),
      port: 3493,
      poll_freq: 30,
      poll_interval: 2,
      max_conn: unsafe { NonZeroUsize::new_unchecked(4) },
      tls_mode: TlsMode::Disable,
    }
  }
}

impl Default for ServerConfig {
  fn default() -> Self {
    Self {
      config_file: None,
      default_theme: None,
      log_level: Level::INFO,
      upsd: Default::default(),
      http_server: Default::default(),
    }
  }
}

impl ServerConfig {
  pub fn new() -> Self {
    Self::default()
  }
}

impl ServerConfig {
  pub fn layer<L>(self, layer: L) -> Self
  where
    L: ConfigLayer,
  {
    layer.apply_layer(self)
  }
}

impl core::fmt::Debug for UpsdConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("UpsdConfig")
      .field("pass", &self.pass.as_ref().map(|_| "******"))
      .field("user", &self.user.as_ref().map(|_| "******"))
      .field("addr", &self.addr)
      .field("port", &self.port)
      .field("poll_freq", &self.poll_freq)
      .field("poll_interval", &self.poll_interval)
      .field("max_conn", &self.max_conn)
      .field("tls_mode", &self.tls_mode)
      .finish()
  }
}
