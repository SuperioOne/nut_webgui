use crate::state::ConnectionStatus;
use nut_webgui_upsmc::{UpsName, ups_status::UpsStatus};
use std::net::IpAddr;

pub mod channel;
pub mod event_batch;

#[derive(Debug, Clone)]
pub struct DeviceStatusChange {
  pub name: UpsName,
  pub old_status: UpsStatus,
  pub new_status: UpsStatus,
}

#[derive(Debug, Clone)]
pub struct DeviceClientInfo {
  pub name: UpsName,
  pub clients: Vec<IpAddr>,
}

#[derive(Debug, Clone)]
pub enum SystemEvent {
  DeviceAddition {
    devices: Vec<UpsName>,
    namespace: Box<str>,
  },
  DeviceRemoval {
    devices: Vec<UpsName>,
    namespace: Box<str>,
  },
  DeviceUpdate {
    devices: Vec<UpsName>,
    namespace: Box<str>,
  },
  DeviceStatusChange {
    changes: Vec<DeviceStatusChange>,
    namespace: Box<str>,
  },
  DaemonStatusUpdate {
    status: ConnectionStatus,
    namespace: Box<str>,
  },
  ClientConnection {
    devices: Vec<DeviceClientInfo>,
    namespace: Box<str>,
  },
  ClientDisconnection {
    devices: Vec<DeviceClientInfo>,
    namespace: Box<str>,
  },
}
