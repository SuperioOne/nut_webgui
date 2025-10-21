use crate::config::{tls_mode::InvalidTlsModeError, uri_path::InvalidPathError};
use core::{net::AddrParseError, num::ParseIntError};
use std::ffi::OsString;
use std::str::ParseBoolError;

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

impl core::error::Error for TomlConfigError {}

impl std::fmt::Display for TomlConfigError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TomlConfigError::IOError { inner } => std::fmt::Display::fmt(&inner, f),
      TomlConfigError::ParseError { inner } => std::fmt::Display::fmt(&inner, f),
      TomlConfigError::InvalidVersion => f.write_str("unknown config toml version"),
    }
  }
}

#[derive(Debug)]
pub enum EnvConfigError {
  IOError { inner: std::io::Error },
  NonUnicodeVar { variable: OsString },
  InvalidNumericFormat { inner: ParseIntError },
  InvalidLogLevelFormat,
  InvalidAddrFormat { inner: core::net::AddrParseError },
  InvalidUriPath,
  InvalidTlsMode,
  InvalidBoolFormat,
}

impl core::error::Error for EnvConfigError {}

impl std::fmt::Display for EnvConfigError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      EnvConfigError::IOError { inner } => {
        f.write_fmt(format_args!("env config io error, {}", inner))
      }
      EnvConfigError::NonUnicodeVar { variable } => f.write_fmt(format_args!(
        "non-unicode variable received, {:?}",
        variable
      )),
      EnvConfigError::InvalidNumericFormat { .. } => f.write_str("invalid numeric format"),
      EnvConfigError::InvalidLogLevelFormat => f.write_str("invalid log level"),
      EnvConfigError::InvalidAddrFormat { inner } => {
        f.write_fmt(format_args!("invalid ip address format, {}", inner))
      }
      EnvConfigError::InvalidUriPath => f.write_str("invalid uri path format"),
      EnvConfigError::InvalidTlsMode => f.write_str("invalid tls mode option"),
      EnvConfigError::InvalidBoolFormat => f.write_str("invalid boolean option"),
    }
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
  fn from(value: ParseIntError) -> Self {
    Self::InvalidNumericFormat { inner: value }
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

impl From<ParseBoolError> for EnvConfigError {
  #[inline]
  fn from(_: ParseBoolError) -> Self {
    Self::InvalidBoolFormat
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
