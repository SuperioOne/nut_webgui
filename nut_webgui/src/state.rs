use crate::device_entry::DeviceEntry;
use chrono::{DateTime, Utc};
use nut_webgui_upsmc::{CmdName, InstCmd, UpsName, VarName};
use serde::Serialize;
use std::{borrow::Borrow, collections::HashMap, time::Instant};

#[derive(Debug)]
pub struct ServerState {
  /// Ups devices
  pub devices: HashMap<UpsName, DeviceEntry>,

  /// NUT daemon sync/connection state
  pub remote_state: DaemonState,

  /// Shared description table for ups variables
  pub shared_desc: HashMap<DescriptionKey, Box<str>>,

  /// Cached command lists
  pub commands_cache: HashMap<UpsName, CommandsCacheEntry>,
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
  pub prot_ver: Option<Box<str>>,
  pub ver: Option<Box<str>>,
}

impl DaemonState {
  pub const fn new() -> DaemonState {
    DaemonState {
      last_device_sync: None,
      status: DaemonStatus::NotReady,
      ver: None,
      prot_ver: None,
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

#[derive(Debug)]
pub struct CommandsCacheEntry {
  pub fetched_at: Instant,
  pub commands: Vec<InstCmd>,
}
