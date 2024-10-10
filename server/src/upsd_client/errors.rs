use crate::upsd_client::ups_variables::UpsError;
use std::fmt::Display;
use std::num::{ParseFloatError, ParseIntError};

#[derive(Debug)]
pub enum NutClientErrors {
  EmptyResponse,
  IOError { kind: std::io::ErrorKind },
  ParseError { kind: ParseErrorKind },
  ProtocolError { kind: UpsError },
}

#[derive(Debug)]
pub enum ParseErrorKind {
  InvalidCmdFormat,
  InvalidListEnd,
  InvalidListStart,
  InvalidUpsFormat,
  InvalidVarFloatFormat { inner: ParseFloatError },
  InvalidVarFormat,
  InvalidVarIntFormat { inner: ParseIntError },
}

impl From<UpsError> for NutClientErrors {
  fn from(kind: UpsError) -> Self {
    Self::ProtocolError { kind }
  }
}

impl From<ParseErrorKind> for NutClientErrors {
  fn from(kind: ParseErrorKind) -> Self {
    Self::ParseError { kind }
  }
}

impl From<std::io::Error> for NutClientErrors {
  fn from(value: std::io::Error) -> Self {
    NutClientErrors::IOError { kind: value.kind() }
  }
}

impl From<ParseIntError> for NutClientErrors {
  #[inline]
  fn from(value: ParseIntError) -> Self {
    Self::ParseError {
      kind: ParseErrorKind::InvalidVarIntFormat { inner: value },
    }
  }
}

impl From<ParseFloatError> for NutClientErrors {
  #[inline]
  fn from(value: ParseFloatError) -> Self {
    Self::ParseError {
      kind: ParseErrorKind::InvalidVarFloatFormat { inner: value },
    }
  }
}

impl Display for NutClientErrors {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      NutClientErrors::EmptyResponse => f.write_str("Empty response received"),
      NutClientErrors::IOError { kind } => f.write_fmt(format_args!("IO error: {}", kind)),
      NutClientErrors::ParseError { kind } => f.write_fmt(format_args!("Parse error: {}", kind)),
      NutClientErrors::ProtocolError { kind } => {
        f.write_fmt(format_args!("NUT protocol error: {}", kind))
      }
    }
  }
}

impl Display for ParseErrorKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let message = match self {
      ParseErrorKind::InvalidCmdFormat => "Malformed CMD line format",
      ParseErrorKind::InvalidUpsFormat => "Malformed UPS line format",
      ParseErrorKind::InvalidVarFloatFormat { inner } => {
        &format!("Invalid float variable: {}", inner)
      }
      ParseErrorKind::InvalidVarFormat => "Malformed VAR line",
      ParseErrorKind::InvalidVarIntFormat { inner } => &format!("Invalid int variable: {}", inner),
      ParseErrorKind::InvalidListEnd => "Invalid list response ending",
      ParseErrorKind::InvalidListStart => "Invalid list response beginning",
    };

    f.write_str(message)
  }
}
