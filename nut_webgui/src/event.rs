use nut_webgui_upsmc::{UpsName, ups_status::UpsStatus};
use tokio::sync::broadcast::{Receiver, Sender, channel};

use crate::state::DaemonStatus;

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
