use chrono::{DateTime, Utc};
use core::net::IpAddr;
use nut_webgui_upsmc::{CmdName, UpsName, VarName, ups_status::UpsStatus, variables::UpsVariables};
use serde::Serialize;
use std::collections::HashMap;

use crate::config::ServerConfig;

#[derive(Debug)]
pub struct ServerState {
  /// Ups devices
  pub devices: HashMap<UpsName, DeviceEntry>,

  /// NUT daemon sync/connection state
  pub state: DaemonState,

  /// Shared description table for ups variables
  pub shared_desc: HashMap<VarName, Box<str>>,
}

#[derive(Debug, Clone)]
pub struct DeviceEntry {
  pub desc: Box<str>,
  pub name: UpsName,
  pub status: UpsStatus,

  pub attached: Vec<IpAddr>,
  pub commands: Vec<CmdName>,
  pub rw_variables: Vec<VarName>,
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
  pub last_full_sync: Option<DateTime<Utc>>,
  pub last_modified: Option<DateTime<Utc>>,
  pub status: DaemonStatus,
}

impl DaemonState {
  pub const fn new() -> DaemonState {
    DaemonState {
      last_full_sync: None,
      last_modified: None,
      status: DaemonStatus::NotReady,
    }
  }
}
