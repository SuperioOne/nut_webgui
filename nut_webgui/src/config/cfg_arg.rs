use super::{ConfigLayer, ServerConfig, utils::override_opt_field};
use crate::config::{
  AuthConfig,
  tls_mode::{InvalidTlsModeError, TlsMode},
  uri_path::{InvalidPathError, UriPath},
};
use clap::Parser;
use core::net::IpAddr;
use std::{num::NonZeroUsize, path::PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct ServerCliArgs {
  /// Non-critical ups variables update frequency in seconds.
  #[arg(long)]
  pub poll_freq: Option<u64>,

  /// Critical ups variables update interval in seconds.
  #[arg(long)]
  pub poll_interval: Option<u64>,

  /// Allowed maximum connection for UPSD client.
  #[arg(long)]
  pub upsd_max_connection: Option<NonZeroUsize>,

  /// NUT server address
  #[arg(long)]
  pub upsd_addr: Option<Box<str>>,

  /// NUT server port
  #[arg(short, long)]
  pub upsd_port: Option<u16>,

  /// NUT username
  #[arg(long)]
  pub upsd_user: Option<Box<str>>,

  /// NUT password
  #[arg(long)]
  pub upsd_pass: Option<Box<str>>,

  /// UPSD connection TLS mode.
  #[arg(long, value_parser =  tls_mode_parser)]
  pub upsd_tls_mode: Option<TlsMode>,

  /// Listen address for HTTP server
  #[arg(short, long)]
  pub listen: Option<IpAddr>,

  /// HTTP server port
  #[arg(short, long)]
  pub port: Option<u16>,

  /// Overrides HTTP server base path
  #[arg(long, value_parser =  uri_path_parser)]
  pub base_path: Option<UriPath>,

  /// Log level
  #[arg(long)]
  pub log_level: Option<tracing::Level>,

  /// Web UI default theme
  #[arg(long)]
  pub default_theme: Option<Box<str>>,

  /// Configuration file
  #[arg(long)]
  pub config_file: Option<PathBuf>,

  /// Enables config override from environment variables
  #[arg(long, default_value_t = false)]
  pub allow_env: bool,

  /// Private server key
  #[arg(long)]
  pub server_key: Option<Box<str>>,

  /// Enables basic auth with users file
  #[arg(long)]
  pub with_auth: Option<PathBuf>,
}

fn uri_path_parser(input: &str) -> Result<UriPath, InvalidPathError> {
  UriPath::new(input)
}

fn tls_mode_parser(input: &str) -> Result<TlsMode, InvalidTlsModeError> {
  input.parse()
}

impl ServerCliArgs {
  /// Alias for [ServerCliArgs::parse()]
  pub fn load() -> Result<Self, clap::Error> {
    Self::try_parse()
  }
}

impl ConfigLayer for ServerCliArgs {
  fn apply_layer(self, mut config: ServerConfig) -> ServerConfig {
    override_opt_field!(config.config_file, self.config_file);
    override_opt_field!(config.default_theme, self.default_theme);
    override_opt_field!(config.log_level, inner_value: self.log_level);
    override_opt_field!(config.server_key, inner_value: self.server_key);

    override_opt_field!(config.upsd.addr, inner_value: self.upsd_addr);
    override_opt_field!(config.upsd.max_conn, inner_value: self.upsd_max_connection);
    override_opt_field!(config.upsd.pass, self.upsd_pass);
    override_opt_field!(config.upsd.poll_freq, inner_value: self.poll_freq);
    override_opt_field!(config.upsd.poll_interval, inner_value: self.poll_interval);
    override_opt_field!(config.upsd.port, inner_value: self.upsd_port);
    override_opt_field!(config.upsd.tls_mode, inner_value: self.upsd_tls_mode);
    override_opt_field!(config.upsd.user, self.upsd_user);

    override_opt_field!(config.http_server.base_path, inner_value:  self.base_path);
    override_opt_field!(config.http_server.listen, inner_value: self.listen);
    override_opt_field!(config.http_server.port, inner_value: self.port);

    if let Some(users_file) = self.with_auth {
      config.auth = Some(AuthConfig { users_file });
    }

    config
  }
}
