use super::{ConfigLayer, ServerConfig, error::TomlConfigError};
use crate::config::{macros::override_opt_field, tls_mode::TlsMode, uri_path::UriPath};
use core::{net::IpAddr, str};
use serde::{Deserialize, de::Visitor};
use std::{fs::File, io::Read, num::NonZeroUsize, path::Path};
use tracing::Level;

#[derive(Debug)]
pub struct LogLevel(tracing::Level);

struct TracingLevelVisitor;

impl From<tracing::Level> for LogLevel {
  fn from(value: tracing::Level) -> Self {
    Self(value)
  }
}

impl<'de> Visitor<'de> for TracingLevelVisitor {
  type Value = LogLevel;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("debug, error, info, warn, trace")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    match v.to_ascii_lowercase().as_ref() {
      "debug" => Ok(Level::DEBUG.into()),
      "error" => Ok(Level::ERROR.into()),
      "warn" => Ok(Level::WARN.into()),
      "info" => Ok(Level::INFO.into()),
      "trace" => Ok(Level::TRACE.into()),
      _ => Err(E::custom(format!("unsupported log level variant: {v}"))),
    }
  }
}

impl<'de> Deserialize<'de> for LogLevel {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_str(TracingLevelVisitor)
  }
}

#[derive(Deserialize, Default, Debug)]
pub struct ServerTomlArgs {
  pub default_theme: Option<Box<str>>,
  pub log_level: Option<LogLevel>,
  pub http_server: HttpServerConfigSection,
  pub upsd: UpsdConfigSection,
}

#[derive(Deserialize, Default, Debug)]
pub struct HttpServerConfigSection {
  pub listen: Option<IpAddr>,
  pub port: Option<u16>,
  pub base_path: Option<UriPath>,
}

#[derive(Deserialize, Default, Debug)]
pub struct UpsdConfigSection {
  pub address: Option<Box<str>>,
  pub password: Option<Box<str>>,
  pub poll_freq: Option<u64>,
  pub poll_interval: Option<u64>,
  pub port: Option<u16>,
  pub username: Option<Box<str>>,
  pub max_connection: Option<NonZeroUsize>,
  pub tls_mode: Option<TlsMode>,
}

impl ServerTomlArgs {
  pub fn load<P>(path: P) -> Result<Self, TomlConfigError>
  where
    P: AsRef<Path>,
  {
    let mut fd = File::open(path)?;
    let mut buffer = String::new();
    _ = fd.read_to_string(&mut buffer)?;

    let deserializer = toml::Deserializer::new(&buffer);
    let config = Self::deserialize(deserializer)?;

    Ok(config)
  }
}

impl ConfigLayer for ServerTomlArgs {
  fn apply_layer(self, mut config: ServerConfig) -> ServerConfig {
    override_opt_field!(config.default_theme, self.default_theme);
    override_opt_field!(config.log_level, inner_value: self.log_level.map(|val| val.0));

    override_opt_field!(config.upsd.addr, inner_value: self.upsd.address);
    override_opt_field!(config.upsd.max_conn, inner_value: self.upsd.max_connection);
    override_opt_field!(config.upsd.pass, self.upsd.password);
    override_opt_field!(config.upsd.poll_freq, inner_value: self.upsd.poll_freq);
    override_opt_field!(config.upsd.poll_interval, inner_value: self.upsd.poll_interval);
    override_opt_field!(config.upsd.port, inner_value: self.upsd.port);
    override_opt_field!(config.upsd.tls_mode, inner_value :self.upsd.tls_mode);
    override_opt_field!(config.upsd.user, self.upsd.username);

    override_opt_field!(config.http_server.base_path, inner_value: self.http_server.base_path);
    override_opt_field!(config.http_server.listen, inner_value: self.http_server.listen);
    override_opt_field!(config.http_server.port, inner_value: self.http_server.port);

    config
  }
}
