use std::net::IpAddr;

use crate::state::DaemonStatus;
use nut_webgui_upsmc::{UpsName, ups_status::UpsStatus};
use tokio::sync::broadcast::{Receiver, Sender, channel};

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
  DeviceAddition { devices: Vec<UpsName> },
  DeviceRemoval { devices: Vec<UpsName> },
  DeviceUpdate { devices: Vec<UpsName> },
  DeviceStatusChange { changes: Vec<DeviceStatusChange> },
  DaemonStatusUpdate { status: DaemonStatus },
  ClientConnection { devices: Vec<DeviceClientInfo> },
  ClientDisconnection { devices: Vec<DeviceClientInfo> },
}

#[derive(Debug)]
pub struct ChannelClosedError;

#[derive(Clone)]
pub struct EventChannel {
  sender: Sender<SystemEvent>,
}

impl EventChannel {
  pub fn new(capacity: usize) -> Self {
    let (sender, _) = channel(capacity);
    Self { sender }
  }

  pub fn subscribe(&self) -> Receiver<SystemEvent> {
    self.sender.subscribe()
  }

  pub fn send(&self, event: SystemEvent) -> Result<(), ChannelClosedError> {
    match self.sender.send(event) {
      Ok(_) => Ok(()),
      Err(_) => Err(ChannelClosedError),
    }
  }
}

impl std::fmt::Display for ChannelClosedError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("channel is closed, or no active listener")
  }
}

impl std::error::Error for ChannelClosedError {}

pub struct EventBatch {
  new: Vec<UpsName>,
  removed: Vec<UpsName>,
  status_changes: Vec<DeviceStatusChange>,
  updated: Vec<UpsName>,
  upsd_status: Option<DaemonStatus>,
  disconnections: Vec<DeviceClientInfo>,
  connections: Vec<DeviceClientInfo>,
}

impl EventBatch {
  pub fn new() -> Self {
    Self {
      new: Vec::new(),
      removed: Vec::new(),
      status_changes: Vec::new(),
      updated: Vec::new(),
      upsd_status: None,
      disconnections: Vec::new(),
      connections: Vec::new(),
    }
  }

  #[inline]
  pub fn new_device(&mut self, name: UpsName) {
    self.new.push(name);
  }

  #[inline]
  pub fn removed_device(&mut self, name: UpsName) {
    self.removed.push(name);
  }

  #[inline]
  pub fn updated_device(&mut self, name: UpsName) {
    self.updated.push(name);
  }

  #[inline]
  pub fn status_change(&mut self, name: UpsName, old_status: UpsStatus, new_status: UpsStatus) {
    self.status_changes.push(DeviceStatusChange {
      name,
      old_status,
      new_status,
    });
  }

  #[inline]
  pub fn client_connection(&mut self, name: UpsName, connected: Vec<IpAddr>) {
    self.connections.push(DeviceClientInfo {
      name,
      clients: connected,
    });
  }

  #[inline]
  pub fn client_disconnect(&mut self, name: UpsName, disconnected: Vec<IpAddr>) {
    self.disconnections.push(DeviceClientInfo {
      name,
      clients: disconnected,
    });
  }

  #[inline]
  pub fn set_upsd_status(&mut self, status: DaemonStatus) {
    self.upsd_status = Some(status);
  }

  pub fn send(self, channel: &EventChannel) -> Result<(), ChannelClosedError> {
    if !self.new.is_empty() {
      channel.send(SystemEvent::DeviceAddition { devices: self.new })?;
    }

    if !self.removed.is_empty() {
      channel.send(SystemEvent::DeviceRemoval {
        devices: self.removed,
      })?;
    }

    if !self.updated.is_empty() {
      channel.send(SystemEvent::DeviceUpdate {
        devices: self.updated,
      })?;
    }

    if !self.status_changes.is_empty() {
      channel.send(SystemEvent::DeviceStatusChange {
        changes: self.status_changes,
      })?;
    }

    if !self.disconnections.is_empty() {
      channel.send(SystemEvent::ClientDisconnection {
        devices: self.disconnections,
      })?;
    }

    if !self.connections.is_empty() {
      channel.send(SystemEvent::ClientConnection {
        devices: self.connections,
      })?;
    }

    if let Some(status) = self.upsd_status {
      channel.send(SystemEvent::DaemonStatusUpdate { status })?;
    }

    Ok(())
  }
}
