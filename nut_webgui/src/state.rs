use chrono::{DateTime, Utc};
use core::net::IpAddr;
use nut_webgui_upsmc::{CmdName, UpsName, VarName, ups_status::UpsStatus, variables::UpsVariables};
use serde::Serialize;
use std::{borrow::Borrow, collections::HashMap};

#[derive(Debug)]
pub struct ServerState {
  /// Ups devices
  pub devices: HashMap<UpsName, DeviceEntry>,

  /// NUT daemon sync/connection state
  pub state: DaemonState,

  /// Shared description table for ups variables
  pub shared_desc: HashMap<DescriptionKey, Box<str>>,
}

#[derive(Debug, Clone)]
pub struct DeviceEntry {
  pub attached: Vec<IpAddr>,
  pub commands: Vec<CmdName>,
  pub desc: Box<str>,
  pub last_modified: DateTime<Utc>,
  pub name: UpsName,
  pub rw_variables: Vec<VarName>,
  pub status: UpsStatus,
  pub variables: UpsVariables,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize)]
pub enum DaemonStatus {
  Dead,
  Online,
  NotReady,
}

impl std::fmt::Display for DaemonStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DaemonStatus::Dead => f.write_str("Dead"),
      DaemonStatus::Online => f.write_str("Online"),
      DaemonStatus::NotReady => f.write_str("Not Ready"),
    }
  }
}

#[derive(Debug)]
pub struct DaemonState {
  pub last_device_sync: Option<DateTime<Utc>>,
  pub status: DaemonStatus,
}

impl DaemonState {
  pub const fn new() -> DaemonState {
    DaemonState {
      last_device_sync: None,
      status: DaemonStatus::NotReady,
    }
  }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct DescriptionKey {
  inner: Box<str>,
}

impl From<CmdName> for DescriptionKey {
  fn from(value: CmdName) -> Self {
    Self {
      inner: Box::from(value.as_str()),
    }
  }
}

impl From<Box<str>> for DescriptionKey {
  fn from(value: Box<str>) -> Self {
    Self { inner: value }
  }
}

impl From<VarName> for DescriptionKey {
  fn from(value: VarName) -> Self {
    Self {
      inner: Box::from(value.as_str()),
    }
  }
}

impl Borrow<str> for DescriptionKey {
  fn borrow(&self) -> &str {
    &self.inner
  }
}
