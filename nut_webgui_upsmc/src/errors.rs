use crate::ups_variables::UpsError;
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
pub enum ParseErrors {
  Empty,
  InvalidChar { position: usize },
  OutOfBounds,
  NaN,
}

#[derive(Debug)]
pub enum ParseErrorKind {
  InvalidName { reason: ParseErrors },
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
      ParseErrorKind::InvalidName { .. } => "Invalid name format",
    };

    f.write_str(message)
  }
}

macro_rules! impl_protocol_errors {
  ({ $(#[$enum_docs:meta])+ } $(
      $(#[$docs:meta])*
      ($variant_name:ident, $value:literal);
    )+
  ) => {

    $(#[$enum_docs])*
    #[derive(Debug, Clone, Eq, PartialEq)]
    pub enum ProtocolErrors {
      Unknown($crate::internal::ReadOnlyStr),
      $(
        $(#[$docs])*
        $variant_name,
      )+
    }

    impl ProtocolErrors {
      pub fn as_str(&self) -> &str {
        match self {
          Self::Unknown(value) => value.as_ref(),
          $(Self::$variant_name => $value,)+
        }
      }
    }

    impl From<&str> for ProtocolErrors {
      fn from(value: &str) -> Self {
        match value {
          $($value => Self::$variant_name,)+
          _ => Self::Unknown(value.into()),
        }
      }
    }
  };
}

impl_protocol_errors!(
  {
    /// UPS protocol errors
  }

  (AccessDenied, "ACCESS-DENIED");
  (AlreadyAttached, "ALREADY-ATTACHED");
  (AlreadySetPassword, "ALREADY-SET-PASSWORD");
  (AlreadySetUsername, "ALREADY-SET-USERNAME");
  (CmdNotSupported, "CMD-NOT-SUPPORTED");
  (DateStale, "DATA-STALE");
  (DriverNotConnected, "DRIVER-NOT-CONNECTED");
  (FeatureNotConfigured, "FEATURE-NOT-CONFIGURED");
  (FeatureNotSupported, "FEATURE-NOT-SUPPORTED");
  (InstcmdFailed, "INSTCMD-FAILED");
  (InvalidArgument, "INVALID-ARGUMENT");
  (InvalidPassword, "INVALID-PASSWORD");
  (InvalidUsername, "INVALID-USERNAME");
  (InvalidValue, "INVALID-VALUE");
  (PasswordRequired, "PASSWORD-REQUIRED");
  (Readonly, "READONLY");
  (SetFailed, "SET-FAILED");
  (TlsAlreadyEnabled, "TLS-ALREADY-ENABLED");
  (TlsNotEnabled, "TLS-NOT-ENABLED");
  (TooLong, "TOO-LONG");
  (UnknownCommand, "UNKNOWN-COMMAND");
  (UnknownUps, "UNKNOWN-UPS");
  (UsernameRequired, "USERNAME-REQUIRED");
  (VarNotSupported, "VAR-NOT-SUPPORTED");
);

impl AsRef<str> for ProtocolErrors {
  #[inline]
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl std::fmt::Display for ProtocolErrors {
  #[inline]
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.as_str())
  }
}

#[cfg(feature = "serde")]
impl serde::Serialize for ProtocolErrors {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(self.as_str())
  }
}
