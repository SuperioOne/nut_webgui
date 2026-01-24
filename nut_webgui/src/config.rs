use crate::config::{tls_mode::TlsMode, uri_path::UriPath, utils::rand_server_key_256bit};
use core::net::{IpAddr, Ipv4Addr};
use std::{collections::HashMap, num::NonZeroUsize, path::PathBuf};
use tracing::level_filters::LevelFilter;

mod utils;

pub mod cfg_arg;
pub mod cfg_env;
pub mod cfg_fallback;
pub mod cfg_toml;
pub mod cfg_user;
pub mod error;
pub mod tls_mode;
pub mod uri_path;

pub const DEFAULT_UPSD_KEY: &str = "default";

pub trait ConfigLayer {
  fn apply_layer(self, config: ServerConfig) -> ServerConfig;
}

pub struct ServerConfig {
  /// Additional config file path
  pub config_file: Option<PathBuf>,

  /// Web GUI default theme
  pub default_theme: Option<Box<str>>,

  /// Logging level
  pub log_level: LevelFilter,

  /// Server instance's private sign key
  pub server_key: Box<str>,

  /// HTTP server configurations
  pub http_server: HttpServerConfig,

  /// UPSD connection configurations
  pub upsd: HashMap<Box<str>, UpsdConfig>,

  /// Authentication scheme configurations
  pub auth: Option<AuthConfig>,
}

#[derive(Debug)]
pub struct HttpServerConfig {
  pub listen: IpAddr,
  pub port: u16,
  pub base_path: UriPath,
  pub worker_count: Option<NonZeroUsize>,
}

#[derive(Clone)]
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

#[derive(Debug)]
pub struct AuthConfig {
  pub users_file: PathBuf,
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
      base_path: UriPath::default(),
      listen: Ipv4Addr::UNSPECIFIED.into(),
      port: 9000,
      worker_count: None,
    }
  }
}

impl Default for UpsdConfig {
  fn default() -> Self {
    Self {
      addr: "localhost".into(),
      max_conn: NonZeroUsize::new(4).expect("static non-zero max_conn is provided as default"),
      pass: None,
      poll_freq: 30,
      poll_interval: 2,
      port: 3493,
      tls_mode: TlsMode::Disable,
      user: None,
    }
  }
}

impl Default for ServerConfig {
  fn default() -> Self {
    Self {
      config_file: None,
      default_theme: None,
      http_server: Default::default(),
      log_level: LevelFilter::INFO,
      server_key: rand_server_key_256bit().into_boxed_str(),
      upsd: Default::default(),
      auth: None,
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

impl core::fmt::Debug for ServerConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ServerConfig")
      .field("config_file", &self.config_file)
      .field("default_theme", &self.default_theme)
      .field("log_level", &self.log_level)
      .field("server_key", &"******")
      .field("http_server", &self.http_server)
      .field("upsd", &self.upsd)
      .field("auth", &self.auth)
      .finish()
  }
}
