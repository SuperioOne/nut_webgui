use crate::config::{tls_mode::InvalidTlsModeError, uri_path::InvalidPathError};
use core::{net::AddrParseError, num::ParseIntError};
use std::ffi::OsString;

#[derive(Debug)]
pub enum ConfigError {
  File(TomlConfigError),
  Environment(EnvConfigError),
  Arguments(clap::Error),
}

#[derive(Debug)]
pub enum TomlConfigError {
  IOError { inner: std::io::Error },
  ParseError { inner: toml::de::Error },
  InvalidVersion,
}

#[derive(Debug)]
pub enum UserTomlError {
  IOError { inner: std::io::Error },
  ParseError { inner: toml::de::Error },
}

#[derive(Debug)]
pub enum EnvConfigError {
  IOError { inner: std::io::Error },
  InvalidAddrFormat { inner: core::net::AddrParseError },
  InvalidLogLevelFormat,
  InvalidNumericFormat,
  InvalidTlsMode,
  InvalidUriPath,
  NonUnicodeVar { variable: OsString },
}

impl std::fmt::Display for EnvConfigError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      EnvConfigError::IOError { inner } => f.write_fmt(format_args!("env config: {}", inner)),
      EnvConfigError::NonUnicodeVar { variable } => f.write_fmt(format_args!(
        "env config: non-unicode variable received, {:?}",
        variable
      )),
      EnvConfigError::InvalidNumericFormat => f.write_str("env config: invalid numeric value"),
      EnvConfigError::InvalidLogLevelFormat => f.write_str("env config: invalid log level"),
      EnvConfigError::InvalidAddrFormat { inner } => {
        f.write_fmt(format_args!("env config: {}", inner))
      }
      EnvConfigError::InvalidUriPath => f.write_str("env config: invalid uri path format"),
      EnvConfigError::InvalidTlsMode => f.write_str("env config: invalid tls mode option"),
    }
  }
}

impl std::fmt::Display for TomlConfigError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::IOError { inner } => f.write_fmt(format_args!("toml config file: {}", inner)),
      Self::ParseError { inner } => f.write_fmt(format_args!("toml config file: {}", inner)),
      Self::InvalidVersion => f.write_str("toml config file: unknown version"),
    }
  }
}

impl std::fmt::Display for UserTomlError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::IOError { inner } => f.write_fmt(format_args!("user toml file: {}", inner)),
      Self::ParseError { inner } => f.write_fmt(format_args!("user toml file: {}", inner)),
    }
  }
}

impl std::fmt::Display for ConfigError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ConfigError::File(e) => e.fmt(f),
      ConfigError::Environment(e) => e.fmt(f),
      ConfigError::Arguments(e) => e.fmt(f),
    }
  }
}

impl From<std::io::Error> for TomlConfigError {
  fn from(value: std::io::Error) -> Self {
    Self::IOError { inner: value }
  }
}

impl From<toml::de::Error> for TomlConfigError {
  fn from(value: toml::de::Error) -> Self {
    Self::ParseError { inner: value }
  }
}

impl From<std::io::Error> for UserTomlError {
  fn from(value: std::io::Error) -> Self {
    Self::IOError { inner: value }
  }
}

impl From<toml::de::Error> for UserTomlError {
  fn from(value: toml::de::Error) -> Self {
    Self::ParseError { inner: value }
  }
}

impl From<EnvConfigError> for ConfigError {
  #[inline]
  fn from(value: EnvConfigError) -> Self {
    Self::Environment(value)
  }
}

impl From<TomlConfigError> for ConfigError {
  #[inline]
  fn from(value: TomlConfigError) -> Self {
    Self::File(value)
  }
}

impl From<clap::Error> for ConfigError {
  #[inline]
  fn from(value: clap::Error) -> Self {
    Self::Arguments(value)
  }
}

impl From<ParseIntError> for EnvConfigError {
  #[inline]
  fn from(_: ParseIntError) -> Self {
    Self::InvalidNumericFormat
  }
}

impl From<InvalidPathError> for EnvConfigError {
  #[inline]
  fn from(_: InvalidPathError) -> Self {
    Self::InvalidUriPath
  }
}

impl From<AddrParseError> for EnvConfigError {
  #[inline]
  fn from(value: AddrParseError) -> Self {
    Self::InvalidAddrFormat { inner: value }
  }
}

impl From<InvalidTlsModeError> for EnvConfigError {
  #[inline]
  fn from(_value: InvalidTlsModeError) -> Self {
    Self::InvalidTlsMode
  }
}

impl From<tracing::metadata::ParseLevelFilterError> for EnvConfigError {
  fn from(_: tracing::metadata::ParseLevelFilterError) -> Self {
    Self::InvalidLogLevelFormat
  }
}

impl From<std::io::Error> for EnvConfigError {
  fn from(value: std::io::Error) -> Self {
    Self::IOError { inner: value }
  }
}

impl core::error::Error for ConfigError {}
impl core::error::Error for EnvConfigError {}
impl core::error::Error for TomlConfigError {}
impl core::error::Error for UserTomlError {}
