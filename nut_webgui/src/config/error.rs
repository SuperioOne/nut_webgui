use crate::uri_path::InvalidPathError;
use core::{net::AddrParseError, num::ParseIntError};
use std::ffi::OsString;

#[derive(Debug)]
pub enum TomlConfigError {
  IOError { inner: std::io::Error },
  ParseError { inner: toml::de::Error },
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

impl std::error::Error for TomlConfigError {}
impl std::fmt::Display for TomlConfigError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TomlConfigError::IOError { inner } => std::fmt::Display::fmt(&inner, f),
      TomlConfigError::ParseError { inner } => std::fmt::Display::fmt(&inner, f),
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
}

impl From<ParseIntError> for EnvConfigError {
  fn from(value: ParseIntError) -> Self {
    Self::InvalidNumericFormat { inner: value }
  }
}

impl From<InvalidPathError> for EnvConfigError {
  fn from(_: InvalidPathError) -> Self {
    Self::InvalidUriPath
  }
}

impl From<AddrParseError> for EnvConfigError {
  fn from(value: AddrParseError) -> Self {
    Self::InvalidAddrFormat { inner: value }
  }
}

impl From<tracing::metadata::ParseLevelError> for EnvConfigError {
  fn from(_: tracing::metadata::ParseLevelError) -> Self {
    Self::InvalidLogLevelFormat
  }
}

impl From<std::io::Error> for EnvConfigError {
  fn from(value: std::io::Error) -> Self {
    Self::IOError { inner: value }
  }
}
