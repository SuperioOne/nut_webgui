use crate::upsd_client::ups_variables::UpsError;

#[derive(Debug)]
pub enum NutClientErrors {
  EmptyResponse,
  IOError(std::io::ErrorKind),
  ParseError(String),
  ProtocolError(UpsError),
}

impl From<std::io::Error> for NutClientErrors {
  fn from(value: std::io::Error) -> Self {
    NutClientErrors::IOError(value.kind())
  }
}
