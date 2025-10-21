use crate::{
  auth::user_store::UserStore,
  config::{ServerConfig, UpsdConfig},
  device_entry::DeviceEntry,
};
use chrono::{DateTime, Utc};
use nut_webgui_upsmc::{CmdName, UpsName, VarName, client::NutPoolClient};
use serde::Serialize;
use std::{borrow::Borrow, collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

/// Server internal state.
pub struct ServerState {
  /// Connected UPSD servers
  pub upsd_servers: HashMap<Box<str>, Arc<UpsdState>>,

  /// Shared description table for ups variables and commands.
  pub shared_desc: RwLock<HashMap<DescriptionKey, Box<str>>>,

  /// Server configurations
  pub config: Arc<ServerConfig>,

  /// Optional user store for authentication
  pub auth_user_store: Option<Arc<UserStore>>,
}

/// Individial UPSD connection state.
pub struct UpsdState {
  /// Daemon state
  pub daemon_state: RwLock<DaemonState>,

  /// Daemon connection pool
  pub connection_pool: NutPoolClient,

  /// Daemon config
  pub config: UpsdConfig,

  /// Upsd namespace
  pub namespace: Box<str>,
}

pub struct DaemonState {
  /// Ups devices
  pub devices: HashMap<UpsName, DeviceEntry>,

  /// Last device sync timestamp
  pub last_device_sync: Option<DateTime<Utc>>,

  /// Daemon connection status
  pub status: ConnectionStatus,

  /// Daemon protocol version
  pub prot_ver: Option<Box<str>>,

  /// Daemon server version
  pub ver: Option<Box<str>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize)]
pub enum ConnectionStatus {
  Dead,
  Online,
  NotReady,
}

impl std::fmt::Display for ConnectionStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ConnectionStatus::Dead => f.write_str("Dead"),
      ConnectionStatus::Online => f.write_str("Online"),
      ConnectionStatus::NotReady => f.write_str("Not Ready"),
    }
  }
}

impl DaemonState {
  pub fn new() -> DaemonState {
    DaemonState {
      devices: HashMap::new(),
      last_device_sync: None,
      prot_ver: None,
      status: ConnectionStatus::NotReady,
      ver: None,
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
