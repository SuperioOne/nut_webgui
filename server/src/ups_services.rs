use crate::upsd_client::ups_variables::UpsVariable;

mod upsd_poll_service;
mod upsd_state_service;

pub use upsd_poll_service::*;
pub use upsd_state_service::*;

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
  FullUpdate { data: Vec<UpsDetails> },

  /// Updates a single variable.
  PartialUpdate { data: Vec<UpsVarDetail> },

  /// Marks daemon as dead
  MarkAsDead,
}
