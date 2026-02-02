use crate::state::{ConnectionStatus, UpsdNamespace};
use nut_webgui_upsmc::{UpsName, ups_status::UpsStatus};
use std::net::IpAddr;

pub mod batch;
pub mod channel;

#[derive(Debug, Clone)]
pub struct DeviceStatusChange {
  pub name: UpsName,
  pub status_old: UpsStatus,
  pub status_new: UpsStatus,
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
    namespace: UpsdNamespace,
  },
  DeviceRemoval {
    devices: Vec<UpsName>,
    namespace: UpsdNamespace,
  },
  DeviceUpdate {
    devices: Vec<UpsName>,
    namespace: UpsdNamespace,
  },
  DeviceStatusChange {
    changes: Vec<DeviceStatusChange>,
    namespace: UpsdNamespace,
  },
  DaemonStatusUpdate {
    status: ConnectionStatus,
    namespace: UpsdNamespace,
  },
  ClientConnection {
    devices: Vec<DeviceClientInfo>,
    namespace: UpsdNamespace,
  },
  ClientDisconnection {
    devices: Vec<DeviceClientInfo>,
    namespace: UpsdNamespace,
  },
}
