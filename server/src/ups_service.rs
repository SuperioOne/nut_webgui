use crate::upsd_client::ups_variables::UpsVariable;

pub mod storage_service;
pub mod ups_poll_service;

#[derive(Debug)]
pub enum UpsUpdateMessage {
  /// Updates all variables.
  FullUpdate {
    name: Box<str>,
    desc: Box<str>,
    commands: Vec<Box<str>>,
    variables: Vec<UpsVariable>,
  },

  /// Updates a single variable.
  PartialUpdate {
    name: Box<str>,
    variable: UpsVariable,
  },
}
