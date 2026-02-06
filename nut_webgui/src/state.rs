use crate::{
  auth::user_store::UserStore,
  config::{ServerConfig, UpsdConfig},
  http::event_api::message_broadcast::MessageBroadcast,
};
use chrono::{DateTime, Utc};
use core::net::IpAddr;
use nut_webgui_upsmc::{
  CmdName, UpsName, Value, VarName, client::NutPoolClient, ups_status::UpsStatus,
  ups_variables::UpsVariables,
};
use serde::{Serialize, ser::SerializeStruct};
use std::{borrow::Borrow, collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

pub type UpsdNamespace = Arc<str>;

/// Server internal state.
pub struct ServerState {
  /// Connected UPSD servers
  pub upsd_servers: HashMap<UpsdNamespace, Arc<UpsdState>>,

  /// Shared description table for ups variables and commands.
  pub shared_desc: RwLock<HashMap<DescriptionKey, Box<str>>>,

  /// Server configurations
  pub config: Arc<ServerConfig>,

  /// Optional user store for authentication
  pub auth_user_store: Option<Arc<UserStore>>,

  pub message_broadcast: MessageBroadcast,
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
  pub namespace: UpsdNamespace,
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

#[derive(Debug, Serialize)]
pub struct DeviceEntry {
  pub attached: Vec<ClientInfo>,
  pub commands: Vec<CmdName>,
  pub desc: Box<str>,
  pub last_modified: DateTime<Utc>,
  pub name: UpsName,
  pub rw_variables: HashMap<VarName, VarDetail>,
  pub status: UpsStatus,
  pub variables: UpsVariables,
}

#[derive(Debug, Serialize)]
pub struct ClientInfo {
  pub addr: IpAddr,
  pub name: Option<Box<str>>,
}

#[derive(Debug, Clone)]
pub enum VarDetail {
  String { max_len: usize },
  Number,
  Enum { options: Vec<Value> },
  Range { min: Value, max: Value },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize)]
pub enum ConnectionStatus {
  Dead,
  Online,
  NotReady,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct DescriptionKey {
  inner: Box<str>,
}

impl Serialize for VarDetail {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    match self {
      VarDetail::String { max_len } => {
        let mut ser = serializer.serialize_struct("VarDetail", 2)?;
        ser.serialize_field("type", "string")?;
        ser.serialize_field("max_len", max_len)?;
        ser.end()
      }
      VarDetail::Number => {
        let mut ser = serializer.serialize_struct("VarDetail", 1)?;
        ser.serialize_field("type", "number")?;
        ser.end()
      }
      VarDetail::Enum { options } => {
        let mut ser = serializer.serialize_struct("VarDetail", 2)?;
        ser.serialize_field("type", "enum")?;
        ser.serialize_field("options", options)?;
        ser.end()
      }
      VarDetail::Range { min, max } => {
        let mut ser = serializer.serialize_struct("VarDetail", 3)?;
        ser.serialize_field("type", "range")?;
        ser.serialize_field("min", min)?;
        ser.serialize_field("max", max)?;
        ser.end()
      }
    }
  }
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

impl DeviceEntry {
  pub fn mark_as_dead_with(&mut self, status: UpsStatus) {
    self.status = status;
    self.commands.clear();
    self.rw_variables.clear();
    self.variables.clear();
    self.last_modified = Utc::now();
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
