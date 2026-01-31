use super::SystemEvent;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender, channel};

#[derive(Clone)]
pub struct EventChannel {
  sender: Sender<Arc<SystemEvent>>,
}

impl EventChannel {
  pub fn new(capacity: usize) -> Self {
    let (sender, _) = channel(capacity);
    Self { sender }
  }

  pub fn subscribe(&self) -> Receiver<Arc<SystemEvent>> {
    self.sender.subscribe()
  }

  pub fn send(&self, event: SystemEvent) -> Result<(), ChannelSendError> {
    let event = Arc::new(event);
    match self.sender.send(event) {
      Ok(_) => Ok(()),
      Err(_) => Err(ChannelSendError),
    }
  }
}

impl std::fmt::Display for ChannelSendError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("channel send event is failed")
  }
}

#[derive(Debug, Copy, Clone)]
pub struct ChannelSendError;

impl std::error::Error for ChannelSendError {}
