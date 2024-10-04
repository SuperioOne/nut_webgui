use crate::upsd_client::ups_variables::UpsVariable;

pub mod storage_service;
pub mod ups_poll_service;

#[derive(Debug)]
pub struct UpsDetails {
  pub name: Box<str>,
  pub desc: Box<str>,
  pub commands: Vec<Box<str>>,
  pub variables: Vec<UpsVariable>,
}

#[derive(Debug)]
pub struct UpsVarDetail {
  pub name: Box<str>,
  pub variable: UpsVariable,
}

#[derive(Debug)]
pub enum UpsUpdateMessage {
  /// Updates all variables.
  FullUpdate { content: Vec<UpsDetails> },

  /// Updates a single variable.
  PartialUpdate { content: Vec<UpsVarDetail> },
}
