use crate::internal::lexer::Position;

#[derive(Debug)]
pub struct Error {
  inner: Box<ErrorKind>,
}

impl Error {
  pub const fn kind(&self) -> &ErrorKind {
    &self.inner
  }
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
  IOError {
    kind: std::io::ErrorKind,
  },
  ParseError {
    inner: ParseError,
    position: Position,
  },
  ProtocolError {
    inner: ProtocolError,
  },
  ConnectionPoolClosed,
  EmptyResponse,
  RequestTimeout,
}

#[derive(Debug, Clone)]
pub enum ParseError {
  CmdName(CmdParseError),
  ExpectedDoubleQuote,
  InvalidToken,
  InvalidIpAddr,
  ExpectedTextToken,
  ExpectedUpsToken,
  ExpectedVarToken,
  ExpectedCmdToken,
  ExpectedDoubleQuotedTextToken,
  InvalidNumber,
  UpsName(UpsNameParseError),
  VarName(VarNameParseError),
  VarType(VarTypeParseError),
}

#[derive(Debug, Clone, Copy)]
pub enum VarTypeParseError {
  Empty,
  InvalidType,
}

#[derive(Debug, Clone, Copy)]
pub enum UpsNameParseError {
  Empty,
  InvalidName,
}

#[derive(Debug, Clone, Copy)]
pub enum VarNameParseError {
  Empty,
  InvalidName,
}

#[derive(Debug, Clone, Copy)]
pub enum CmdParseError {
  Empty,
  InvalidName,
}

#[derive(Debug, Clone, Copy)]
pub struct NumberParseError;

impl std::fmt::Display for CmdParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      CmdParseError::Empty => f.write_str("empty string received"),
      CmdParseError::InvalidName => f.write_str("invalid command name"),
    }
  }
}

impl std::fmt::Display for VarNameParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      VarNameParseError::Empty => f.write_str("empty string received"),
      VarNameParseError::InvalidName => f.write_str("invalid variable name"),
    }
  }
}

impl std::fmt::Display for UpsNameParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      UpsNameParseError::Empty => f.write_str("empty string received"),
      UpsNameParseError::InvalidName => f.write_str("invalid UPS name"),
    }
  }
}

impl std::fmt::Display for VarTypeParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      VarTypeParseError::Empty => f.write_str("empty string received"),
      VarTypeParseError::InvalidType => f.write_str("invalid type"),
    }
  }
}

impl std::fmt::Display for NumberParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("parse failed not a valid number neither f64 nor i64")
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.inner.fmt(f)
  }
}

impl std::fmt::Display for ErrorKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ErrorKind::RequestTimeout => f.write_str("request timeout"),
      ErrorKind::EmptyResponse => f.write_str("empty response received"),
      ErrorKind::ConnectionPoolClosed => {
        f.write_str("new connection request received but connection pool is already closed")
      }
      ErrorKind::IOError { kind } => f.write_fmt(format_args!("io error occured. kind={}", kind)),
      ErrorKind::ParseError { inner, position } => f.write_fmt(format_args!(
        "{} at {}:{}",
        inner, position.line, position.col
      )),
      ErrorKind::ProtocolError { inner } => f.write_fmt(format_args!(
        "nut server responded with an error message. error={}",
        inner
      )),
    }
  }
}

impl std::fmt::Display for ParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ParseError::CmdName(inner) => inner.fmt(f),
      ParseError::ExpectedCmdToken => f.write_str("expected command token"),
      ParseError::ExpectedDoubleQuote => f.write_str("expected double quote character"),
      ParseError::ExpectedDoubleQuotedTextToken => f.write_str("expected double quoted text token"),
      ParseError::ExpectedTextToken => f.write_str("expected text token"),
      ParseError::ExpectedUpsToken => f.write_str("expected ups token"),
      ParseError::ExpectedVarToken => f.write_str("expected var token"),
      ParseError::InvalidIpAddr => f.write_str("invalid ipv4 or ipv6 text"),
      ParseError::InvalidToken => f.write_str("invalid token"),
      ParseError::InvalidNumber => f.write_str("invalid number value token"),
      ParseError::UpsName(inner) => inner.fmt(f),
      ParseError::VarName(inner) => inner.fmt(f),
      ParseError::VarType(inner) => inner.fmt(f),
    }
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
    pub enum ProtocolError {
      Unknown(Box<str>),
      $(
        $(#[$docs])*
        $variant_name,
      )+
    }

    impl ProtocolError {
      pub fn as_str(&self) -> &str {
        match self {
          Self::Unknown(value) => value.as_ref(),
          $(Self::$variant_name => $value,)+
        }
      }
    }

    impl From<&str> for ProtocolError {
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

impl AsRef<str> for ProtocolError {
  #[inline]
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl std::fmt::Display for ProtocolError {
  #[inline]
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.as_str())
  }
}

#[cfg(feature = "serde")]
impl serde::Serialize for ProtocolError {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(self.as_str())
  }
}

impl From<ProtocolError> for Error {
  fn from(value: ProtocolError) -> Self {
    Self {
      inner: Box::from(ErrorKind::ProtocolError { inner: value }),
    }
  }
}

impl From<ErrorKind> for Error {
  fn from(value: ErrorKind) -> Self {
    Self {
      inner: Box::from(value),
    }
  }
}

impl From<std::io::ErrorKind> for Error {
  fn from(value: std::io::ErrorKind) -> Self {
    Self {
      inner: Box::from(ErrorKind::IOError { kind: value }),
    }
  }
}

impl From<std::io::Error> for Error {
  fn from(value: std::io::Error) -> Self {
    Self {
      inner: Box::from(ErrorKind::IOError { kind: value.kind() }),
    }
  }
}

impl core::error::Error for CmdParseError {}
impl core::error::Error for Error {}
impl core::error::Error for ParseError {}
impl core::error::Error for UpsNameParseError {}
impl core::error::Error for VarNameParseError {}
impl core::error::Error for NumberParseError {}
impl core::error::Error for VarTypeParseError {}
