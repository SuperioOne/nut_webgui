use super::{
  DeviceClientInfo, DeviceStatusChange, SystemEvent,
  channel::{ChannelSendError, EventChannel},
};
use crate::state::{ConnectionStatus, UpsdNamespace};
use nut_webgui_upsmc::{UpsName, ups_status::UpsStatus};
use std::net::IpAddr;

/// This struct does not exactly send all events as a single message, but simply aggregates all events in
/// one place.
pub struct EventBatch {
  namespace: UpsdNamespace,
  upsd_status: Option<ConnectionStatus>,
  new: Vec<UpsName>,
  removed: Vec<UpsName>,
  status_changes: Vec<DeviceStatusChange>,
  updated: Vec<UpsName>,
  disconnections: Vec<DeviceClientInfo>,
  connections: Vec<DeviceClientInfo>,
}

impl EventBatch {
  pub const fn new(namespace: UpsdNamespace) -> Self {
    Self {
      namespace,
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
      status_old: old_status,
      status_new: new_status,
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
  pub const fn set_upsd_status(&mut self, status: ConnectionStatus) {
    self.upsd_status = Some(status);
  }

  pub fn send(self, channel: &EventChannel) -> Result<(), ChannelSendError> {
    if !self.new.is_empty() {
      channel.send(SystemEvent::DeviceAddition {
        devices: self.new,
        namespace: self.namespace.clone(),
      })?;
    }

    if !self.removed.is_empty() {
      channel.send(SystemEvent::DeviceRemoval {
        devices: self.removed,
        namespace: self.namespace.clone(),
      })?;
    }

    if !self.updated.is_empty() {
      channel.send(SystemEvent::DeviceUpdate {
        devices: self.updated,
        namespace: self.namespace.clone(),
      })?;
    }

    if !self.status_changes.is_empty() {
      channel.send(SystemEvent::DeviceStatusChange {
        changes: self.status_changes,
        namespace: self.namespace.clone(),
      })?;
    }

    if !self.disconnections.is_empty() {
      channel.send(SystemEvent::ClientDisconnection {
        devices: self.disconnections,
        namespace: self.namespace.clone(),
      })?;
    }

    if !self.connections.is_empty() {
      channel.send(SystemEvent::ClientConnection {
        devices: self.connections,
        namespace: self.namespace.clone(),
      })?;
    }

    if let Some(status) = self.upsd_status {
      channel.send(SystemEvent::DaemonStatusUpdate {
        status,
        namespace: self.namespace.clone(),
      })?;
    }

    Ok(())
  }
}
