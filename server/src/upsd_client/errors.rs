use std::io;
use crate::upsd_client::protocol::UpsError;

#[derive(Debug)]
pub enum NutClientErrors {
  EmptyResponse,
  IOError(io::ErrorKind),
  ParseError(String),
  ProtocolError(UpsError),
}