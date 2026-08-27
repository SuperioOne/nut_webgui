use super::{
  ConfigLayer, ServerConfig,
  error::{ConfigError, ParsePathError},
  uri_path::UriPath,
  utils::override_opt_field,
};
use clap::Parser;
use core::net::IpAddr;
use std::{fs, num::NonZeroUsize, path::PathBuf};
use tracing::warn;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct ServerCliArgs {
  /// Listen address for HTTP server
  #[arg(short, long)]
  pub listen: Option<IpAddr>,

  /// HTTP server port
  #[arg(short, long)]
  pub port: Option<u16>,

  /// Override HTTP server base path
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

  /// Enable config override from environment variables
  #[arg(long, default_value_t = false)]
  pub allow_env: bool,

  /// Allow/Disallow anonymous access to metrics endpoint
  #[arg(long)]
  pub anonymous_metrics: Option<bool>,

  /// Set private server key
  #[arg(long)]
  pub server_key: Option<Box<str>>,

  /// Set private server key from file
  #[arg(long)]
  pub server_key_file: Option<PathBuf>,

  /// Enable basic auth with users file
  #[arg(long)]
  pub with_auth: Option<PathBuf>,

  /// HTTP server worker count, default is all available system CPU count.
  #[arg(short, long)]
  pub worker_count: Option<NonZeroUsize>,
}

fn uri_path_parser(input: &str) -> Result<UriPath, ParsePathError> {
  UriPath::new(input)
}

impl ServerCliArgs {
  #[inline]
  pub fn new() -> Result<Self, clap::error::Error> {
    Self::try_parse()
  }
}

impl ConfigLayer for ServerCliArgs {
  fn apply_layer(self, mut config: ServerConfig) -> Result<ServerConfig, ConfigError> {
    override_opt_field!(config.config_file, self.config_file);
    override_opt_field!(config.default_theme, self.default_theme);
    override_opt_field!(config.log_level, inner_value: self.log_level);

    override_opt_field!(config.http_server.base_path, inner_value:  self.base_path);
    override_opt_field!(config.http_server.listen, inner_value: self.listen);
    override_opt_field!(config.http_server.port, inner_value: self.port);
    override_opt_field!(config.http_server.worker_count, self.worker_count);

    override_opt_field!(config.auth.users_file, self.with_auth);
    override_opt_field!(
      config.auth.allow_anonymous_metrics,
      inner_value: self.anonymous_metrics
    );

    if let Some(path) = self.server_key_file.as_ref() {
      let key = fs::read_to_string(path)
        .map_err(|err| ConfigError::ServerKeyIOError { inner: err })?
        .into_boxed_str();

      if key.is_empty() {
        return Err(ConfigError::EmptyServerKey);
      } else if self.server_key.is_some_and(|v| !v.is_empty()) {
        warn!(
          "both server_key and server_key_file options are set, server uses the key file as the default"
        );
      }

      config.server_key = key.into_boxed_bytes();
    } else {
      override_opt_field!(config.server_key, inner_value: self.server_key.map(|v| v.into_boxed_bytes()));
    }

    Ok(config)
  }
}
