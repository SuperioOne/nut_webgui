use core::{net::AddrParseError, num::ParseIntError};
use std::ffi::OsString;

#[derive(Debug)]
pub enum UserTomlError {
  IOError { inner: std::io::Error },
  ParseTomlError { inner: toml::de::Error },
}

#[derive(Debug, Clone, Copy)]
pub struct ParseTlsModeError;

#[derive(Debug, Clone, Copy)]
pub struct ParsePathError;

#[derive(Debug)]
pub enum ConfigError {
  UnsupportedVersion,
  EmptyServerKey,
  ArgumentError {
    inner: clap::Error,
  },
  NonUnicodeEnvironmentVariable {
    variable: OsString,
  },
  ParseAddrError {
    inner: core::net::AddrParseError,
  },
  ParseIntError {
    inner: core::num::ParseIntError,
  },
  ParseLogLevelError {
    inner: tracing::metadata::ParseLevelFilterError,
  },
  ParseTlsModeError {
    inner: ParseTlsModeError,
  },
  ParseTomlError {
    inner: toml::de::Error,
  },
  ParsePathError {
    inner: ParsePathError,
  },
  ServerKeyIOError {
    inner: std::io::Error,
  },
  ConfigFileIOError {
    inner: std::io::Error,
  },
  EnvFileIOError {
    inner: std::io::Error,
    variable: String,
  },
}

impl std::fmt::Display for ParsePathError {
  #[inline]
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("invalid base path value")
  }
}

impl core::fmt::Display for ParseTlsModeError {
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str("invalid tls mode option")
  }
}

impl std::fmt::Display for UserTomlError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::IOError { inner } => f.write_fmt(format_args!("user toml file: {}", inner)),
      Self::ParseTomlError { inner } => f.write_fmt(format_args!("user toml file: {}", inner)),
    }
  }
}

impl std::fmt::Display for ConfigError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ConfigError::ArgumentError { inner } => inner.fmt(f),
      ConfigError::ParseAddrError { inner } => inner.fmt(f),
      ConfigError::ParseTomlError { inner } => inner.fmt(f),
      ConfigError::UnsupportedVersion => f.write_str("unsupported configuration scheme version"),
      ConfigError::ParseIntError { inner } => inner.fmt(f),
      ConfigError::ParseLogLevelError { inner } => inner.fmt(f),
      ConfigError::ParseTlsModeError { inner } => inner.fmt(f),
      ConfigError::ParsePathError { inner } => inner.fmt(f),
      ConfigError::NonUnicodeEnvironmentVariable { variable } => f.write_fmt(format_args!(
        "unable to read {} environment variable, it contains non-unicode characters",
        variable.display()
      )),
      ConfigError::EmptyServerKey => f.write_str("server key value is empty"),
      ConfigError::ServerKeyIOError { inner } => {
        f.write_fmt(format_args!("unable to read server key file, {}", inner))
      }
      ConfigError::ConfigFileIOError { inner } => {
        f.write_fmt(format_args!("unable to read toml file, {}", inner))
      }
      ConfigError::EnvFileIOError { inner, variable } => f.write_fmt(format_args!(
        "unable to read target file path from environment variable {}, {}",
        variable, inner
      )),
    }
  }
}

impl From<toml::de::Error> for ConfigError {
  #[inline]
  fn from(value: toml::de::Error) -> Self {
    Self::ParseTomlError { inner: value }
  }
}

impl From<clap::Error> for ConfigError {
  #[inline]
  fn from(value: clap::Error) -> Self {
    Self::ArgumentError { inner: value }
  }
}

impl From<ParseIntError> for ConfigError {
  #[inline]
  fn from(value: ParseIntError) -> Self {
    Self::ParseIntError { inner: value }
  }
}

impl From<ParsePathError> for ConfigError {
  #[inline]
  fn from(value: ParsePathError) -> Self {
    Self::ParsePathError { inner: value }
  }
}

impl From<AddrParseError> for ConfigError {
  #[inline]
  fn from(value: AddrParseError) -> Self {
    Self::ParseAddrError { inner: value }
  }
}

impl From<ParseTlsModeError> for ConfigError {
  #[inline]
  fn from(value: ParseTlsModeError) -> Self {
    Self::ParseTlsModeError { inner: value }
  }
}

impl From<tracing::metadata::ParseLevelFilterError> for ConfigError {
  #[inline]
  fn from(value: tracing::metadata::ParseLevelFilterError) -> Self {
    Self::ParseLogLevelError { inner: value }
  }
}

impl From<std::io::Error> for UserTomlError {
  #[inline]
  fn from(value: std::io::Error) -> Self {
    Self::IOError { inner: value }
  }
}

impl From<toml::de::Error> for UserTomlError {
  #[inline]
  fn from(value: toml::de::Error) -> Self {
    Self::ParseTomlError { inner: value }
  }
}

impl core::error::Error for ConfigError {}
impl core::error::Error for UserTomlError {}
impl core::error::Error for ParseTlsModeError {}
impl std::error::Error for ParsePathError {}
