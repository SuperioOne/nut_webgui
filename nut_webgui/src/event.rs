use crate::state::DaemonStatus;
use nut_webgui_upsmc::{UpsName, ups_status::UpsStatus};
use tokio::sync::broadcast::{Receiver, Sender, channel};

#[derive(Debug, Clone)]
pub struct UpsStatusDetails {
  pub name: UpsName,
  pub old_status: UpsStatus,
  pub new_status: UpsStatus,
}

#[derive(Debug, Clone)]
pub enum SystemEvent {
  DevicesAdded { devices: Vec<UpsName> },
  DevicesRemoved { devices: Vec<UpsName> },
  DevicesUpdated { devices: Vec<UpsName> },
  DeviceStatusUpdates { changes: Vec<UpsStatusDetails> },
  UpsdStatus { status: DaemonStatus },
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
  updated: Vec<UpsName>,
  status_changes: Vec<UpsStatusDetails>,
  upsd_status: Option<DaemonStatus>,
}

impl EventBatch {
  pub fn new() -> Self {
    Self {
      upsd_status: None,
      new: Vec::new(),
      updated: Vec::new(),
      removed: Vec::new(),
      status_changes: Vec::new(),
    }
  }

  pub fn push_new_device(&mut self, name: UpsName) {
    self.new.push(name);
  }

  pub fn push_removed_device(&mut self, name: UpsName) {
    self.removed.push(name);
  }

  pub fn push_updated_device(&mut self, name: UpsName) {
    self.updated.push(name);
  }

  pub fn push_status_change(&mut self, change: UpsStatusDetails) {
    self.status_changes.push(change);
  }

  pub fn set_upsd_status(&mut self, status: DaemonStatus) {
    self.upsd_status = Some(status);
  }

  pub fn send(self, channel: &EventChannel) -> Result<(), ChannelClosedError> {
    if !self.new.is_empty() {
      channel.send(SystemEvent::DevicesAdded { devices: self.new })?;
    }

    if !self.removed.is_empty() {
      channel.send(SystemEvent::DevicesRemoved {
        devices: self.removed,
      })?;
    }

    if !self.updated.is_empty() {
      channel.send(SystemEvent::DevicesUpdated {
        devices: self.updated,
      })?;
    }

    if !self.status_changes.is_empty() {
      channel.send(SystemEvent::DeviceStatusUpdates {
        changes: self.status_changes,
      })?;
    }

    if let Some(status) = self.upsd_status {
      channel.send(SystemEvent::UpsdStatus { status })?;
    }

    Ok(())
  }
}
