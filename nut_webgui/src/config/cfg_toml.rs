use crate::config::{
  AuthConfig, ConfigLayer, ServerConfig, UpsdConfig, error::TomlConfigError, tls_mode::TlsMode,
  uri_path::UriPath, utils::override_opt_field,
};
use core::{net::IpAddr, str};
use serde::{Deserialize, de::Visitor};
use std::{
  collections::HashMap,
  fs::File,
  io::Read,
  num::NonZeroUsize,
  path::{Path, PathBuf},
};
use toml::Table;
use tracing::level_filters::LevelFilter;

#[derive(Debug)]
pub struct LogLevel(LevelFilter);

struct TracingLevelVisitor;

impl From<tracing::level_filters::LevelFilter> for LogLevel {
  fn from(value: tracing::level_filters::LevelFilter) -> Self {
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
      "debug" => Ok(LevelFilter::DEBUG.into()),
      "error" => Ok(LevelFilter::ERROR.into()),
      "warn" => Ok(LevelFilter::WARN.into()),
      "info" => Ok(LevelFilter::INFO.into()),
      "trace" => Ok(LevelFilter::TRACE.into()),
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

#[derive(Debug, Deserialize, Default)]
pub struct ServerTomlArgs {
  pub default_theme: Option<Box<str>>,
  pub log_level: Option<LogLevel>,
  pub http_server: Option<HttpServerConfigSection>,
  pub upsd: Option<HashMap<Box<str>, UpsdConfigSection>>,
  pub auth: Option<AuthConfigSection>,
}

#[derive(Deserialize, Default, Debug)]
pub struct HttpServerConfigSection {
  pub listen: Option<IpAddr>,
  pub port: Option<u16>,
  pub base_path: Option<UriPath>,
  pub worker_count: Option<NonZeroUsize>,
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

#[derive(Deserialize, Default, Debug)]
pub struct AuthConfigSection {
  users_file: PathBuf,
}

impl ServerTomlArgs {
  pub fn load<P>(path: P) -> Result<Self, TomlConfigError>
  where
    P: AsRef<Path>,
  {
    let mut fd = File::open(path)?;
    let mut buffer = String::new();
    _ = fd.read_to_string(&mut buffer)?;

    let deserializer = toml::Deserializer::parse(&buffer)?;
    let root = Table::deserialize(deserializer)?;

    match root.get("version") {
      Some(toml::Value::String(version)) if version == "1" => {
        root.try_into::<ServerTomlArgs>().map_err(|err| err.into())
      }
      _ => Err(TomlConfigError::InvalidVersion),
    }
  }
}

impl ConfigLayer for ServerTomlArgs {
  fn apply_layer(self, mut config: ServerConfig) -> ServerConfig {
    override_opt_field!(config.default_theme, self.default_theme);
    override_opt_field!(config.log_level, inner_value: self.log_level.map(|val| val.0));

    if let Some(upsd_section) = self.upsd
      && !upsd_section.is_empty()
    {
      for (key, val) in upsd_section.into_iter() {
        let mut upsd_cfg = UpsdConfig::default();

        override_opt_field!(upsd_cfg.addr, inner_value: val.address);
        override_opt_field!(upsd_cfg.max_conn, inner_value: val.max_connection);
        override_opt_field!(upsd_cfg.pass, val.password);
        override_opt_field!(upsd_cfg.poll_freq, inner_value: val.poll_freq);
        override_opt_field!(upsd_cfg.poll_interval, inner_value: val.poll_interval);
        override_opt_field!(upsd_cfg.port, inner_value: val.port);
        override_opt_field!(upsd_cfg.tls_mode, inner_value : val.tls_mode);
        override_opt_field!(upsd_cfg.user, val.username);

        config.upsd.insert(key, upsd_cfg);
      }
    }

    if let Some(http_server) = self.http_server {
      override_opt_field!(config.http_server.base_path, inner_value: http_server.base_path);
      override_opt_field!(config.http_server.listen, inner_value: http_server.listen);
      override_opt_field!(config.http_server.port, inner_value: http_server.port);
      override_opt_field!(config.http_server.worker_count, http_server.worker_count);
    }

    if let Some(auth) = self.auth {
      config.auth = Some(AuthConfig {
        users_file: auth.users_file,
      })
    }

    config
  }
}
