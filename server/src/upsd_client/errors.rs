use crate::upsd_client::protocol::UpsError;
use std::io;

#[derive(Debug)]
pub enum NutClientErrors {
  EmptyResponse,
  IOError(io::ErrorKind),
  ParseError(String),
  ProtocolError(UpsError),
}

impl From<io::Error> for NutClientErrors {
  fn from(value: io::Error) -> Self {
    NutClientErrors::IOError(value.kind())
  }
}
