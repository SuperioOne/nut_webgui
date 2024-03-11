use crate::upsd_client::ups_variables::UpsVariable;

pub mod storage_service;
pub mod ups_poll_service;

#[derive(Debug)]
pub struct UpsUpdateMessage {
  name: Box<str>,
  desc: Box<str>,
  commands: Vec<Box<str>>,
  variables: Vec<UpsVariable>,
}
