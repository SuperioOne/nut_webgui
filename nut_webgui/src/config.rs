use crate::uri_path::UriPath;
use core::net::{IpAddr, Ipv4Addr};
use serde::{Deserialize, Serialize};
use std::{num::NonZeroUsize, path::PathBuf};
use tracing::Level;

pub mod cfg_args;
pub mod cfg_env;
pub mod cfg_toml;
pub mod error;
mod macros;

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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpsdTlsMode {
  Disabled,
  Enabled,
  EnabledSkipVerify,
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
  pub tls_mode: UpsdTlsMode,
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
      tls_mode: UpsdTlsMode::Disabled,
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
    #[derive(Debug)]
    struct _Filtered<'a> {
      poll_freq: u64,
      poll_interval: u64,
      addr: &'a str,
      port: u16,
      user: Option<&'static str>,
      pass: Option<&'static str>,
      max_conn: NonZeroUsize,
      tls_mode: UpsdTlsMode,
    }

    let filtered = _Filtered {
      poll_freq: self.poll_freq,
      poll_interval: self.poll_interval,
      addr: self.addr.as_ref(),
      port: self.port,
      user: if self.user.is_some() {
        Some("******")
      } else {
        None
      },
      pass: if self.pass.is_some() {
        Some("******")
      } else {
        None
      },
      max_conn: self.max_conn,
      tls_mode: self.tls_mode,
    };

    core::fmt::Debug::fmt(&filtered, f)
  }
}
