use super::{ConfigLayer, ServerConfig, error::EnvConfigError};
use crate::config::{
  AuthConfig, DEFAULT_UPSD_KEY, TlsMode, UpsdConfig, UriPath, utils::override_opt_field,
};
use core::net::IpAddr;
use std::{
  env,
  fs::File,
  io::Read,
  num::NonZeroUsize,
  path::{Path, PathBuf},
};
use tracing::level_filters::LevelFilter;

#[derive(Debug, Default)]
pub struct ServerEnvArgs {
  pub auth_users_file: Option<PathBuf>,
  pub base_path: Option<UriPath>,
  pub config_file: Option<PathBuf>,
  pub default_theme: Option<Box<str>>,
  pub listen: Option<IpAddr>,
  pub log_level: Option<tracing::level_filters::LevelFilter>,
  pub poll_freq: Option<u64>,
  pub poll_interval: Option<u64>,
  pub port: Option<u16>,
  pub server_key: Option<Box<str>>,
  pub upsd_name: Option<Box<str>>,
  pub upsd_addr: Option<Box<str>>,
  pub upsd_max_conn: Option<NonZeroUsize>,
  pub upsd_pass: Option<Box<str>>,
  pub upsd_port: Option<u16>,
  pub upsd_tls: Option<TlsMode>,
  pub upsd_user: Option<Box<str>>,
}

fn load_from_env(key: &str) -> Result<Option<String>, EnvConfigError> {
  match env::var(key) {
    Ok(value) => {
      if value.is_empty() {
        Ok(None)
      } else {
        let path = Path::new(&value);

        if path.is_file() {
          let mut buffer = String::new();
          let mut fd = File::open(path)?;
          _ = fd.read_to_string(&mut buffer)?;

          Ok(Some(buffer))
        } else {
          Ok(Some(value))
        }
      }
    }
    Err(env::VarError::NotUnicode(variable)) => Err(EnvConfigError::NonUnicodeVar { variable }),
    Err(env::VarError::NotPresent) => Ok(None),
  }
}

macro_rules! load_var {
  ($(($env_name:literal, $target_field:expr, $output_type:tt);)+) => {
    $(
      load_var!(@rule $env_name, $target_field, $output_type);
    )+
  };

  (@rule $env_name:literal, $target_field:expr, boxed_str) => {
    if let Some(value) = $crate::config::cfg_env::load_from_env($env_name)? {
      $target_field = Some(Box::from(value.trim()));
    }
  };
  (@rule $env_name:literal, $target_field:expr, path_buf) => {
    if let Ok(value) = std::env::var($env_name) {
      let path_str = value.trim();
      if path_str.is_empty() {
        $target_field = None;
      } else {
        $target_field = Some(std::path::PathBuf::from(path_str));
      }
    }
  };
    (@rule $env_name:literal, $target_field:expr, $other_type:ty) => {
    if let Some(value) = $crate::config::cfg_env::load_from_env($env_name)? {
      $target_field = Some(value.trim().parse::<$other_type>()?);
    }
  };
}

impl ServerEnvArgs {
  pub fn load() -> Result<Self, EnvConfigError> {
    let mut env_config = Self::default();

    load_var!(
      ("NUTWG__CONFIG_FILE",             env_config.config_file,     path_buf);
      ("NUTWG__DEFAULT_THEME",           env_config.default_theme,   boxed_str);
      ("NUTWG__LOG_LEVEL",               env_config.log_level,       LevelFilter);
      ("NUTWG__SERVER_KEY",              env_config.server_key,      boxed_str);

      ("NUTWG__HTTP_SERVER__BASE_PATH",  env_config.base_path,       UriPath);
      ("NUTWG__HTTP_SERVER__LISTEN",     env_config.listen,          IpAddr);
      ("NUTWG__HTTP_SERVER__PORT",       env_config.port,            u16);

      ("NUTWG__AUTH__USERS_FILE",        env_config.auth_users_file, path_buf);

      ("NUTWG__UPSD__NAME",              env_config.upsd_name,       boxed_str);
      ("NUTWG__UPSD__ADDRESS",           env_config.upsd_addr,       boxed_str);
      ("NUTWG__UPSD__MAX_CONNECTION",    env_config.upsd_max_conn,   NonZeroUsize);
      ("NUTWG__UPSD__PASSWORD",          env_config.upsd_pass,       boxed_str);
      ("NUTWG__UPSD__POLL_FREQ",         env_config.poll_freq,       u64);
      ("NUTWG__UPSD__POLL_INTERVAL",     env_config.poll_interval,   u64);
      ("NUTWG__UPSD__PORT",              env_config.upsd_port,       u16);
      ("NUTWG__UPSD__TLS_MODE",          env_config.upsd_tls,        TlsMode);
      ("NUTWG__UPSD__USERNAME",          env_config.upsd_user,       boxed_str);
    );

    Ok(env_config)
  }
}

impl ConfigLayer for ServerEnvArgs {
  fn apply_layer(self, mut config: ServerConfig) -> ServerConfig {
    override_opt_field!(config.config_file, self.config_file);
    override_opt_field!(config.default_theme, self.default_theme);
    override_opt_field!(config.log_level, inner_value: self.log_level);
    override_opt_field!(config.server_key, inner_value: self.server_key);

    override_opt_field!(config.http_server.base_path, inner_value: self.base_path);
    override_opt_field!(config.http_server.listen, inner_value: self.listen);
    override_opt_field!(config.http_server.port, inner_value: self.port);

    if let Some(users_file) = self.auth_users_file {
      config.auth = Some(AuthConfig { users_file });
    }

    if self.upsd_addr.is_some() {
      let namespace = self
        .upsd_name
        .as_ref()
        .map_or_else(|| Box::from(DEFAULT_UPSD_KEY), |v| v.clone());

      config.upsd.insert(namespace, UpsdConfig::default());
    }

    let key: &str = self
      .upsd_name
      .as_ref()
      .map_or(DEFAULT_UPSD_KEY, |v| v.as_ref());

    if let Some(default_upsd) = config.upsd.get_mut(key) {
      override_opt_field!(default_upsd.addr, inner_value: self.upsd_addr);
      override_opt_field!(default_upsd.max_conn, inner_value: self.upsd_max_conn);
      override_opt_field!(default_upsd.pass, self.upsd_pass);
      override_opt_field!(default_upsd.poll_freq, inner_value: self.poll_freq);
      override_opt_field!(default_upsd.poll_interval, inner_value: self.poll_interval);
      override_opt_field!(default_upsd.port, inner_value: self.upsd_port);
      override_opt_field!(default_upsd.tls_mode, inner_value: self.upsd_tls);
      override_opt_field!(default_upsd.user, self.upsd_user);
    }

    config
  }
}
