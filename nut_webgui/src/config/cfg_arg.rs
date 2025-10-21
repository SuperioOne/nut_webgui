use super::{ConfigLayer, ServerConfig, utils::override_opt_field};
use crate::config::{
  AuthConfig,
  uri_path::{InvalidPathError, UriPath},
};
use clap::Parser;
use core::net::IpAddr;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct ServerCliArgs {
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
  pub log_level: Option<tracing::level_filters::LevelFilter>,

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

    override_opt_field!(config.http_server.base_path, inner_value:  self.base_path);
    override_opt_field!(config.http_server.listen, inner_value: self.listen);
    override_opt_field!(config.http_server.port, inner_value: self.port);

    if let Some(users_file) = self.with_auth {
      config.auth = Some(AuthConfig { users_file });
    }

    config
  }
}
