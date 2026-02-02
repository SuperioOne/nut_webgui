use super::message::NutEventMessage;
use crate::{
  background_service::BackgroundService,
  event::{DeviceClientInfo, DeviceStatusChange, SystemEvent, channel::EventChannel},
  state::ConnectionStatus,
};
use chrono::Utc;
use nut_webgui_upsmc::{UpsName, ups_event::UpsEvents};
use std::sync::Arc;
use tokio::{
  select,
  sync::broadcast::{Sender, error::RecvError},
};
use tokio_util::sync::CancellationToken;
use tracing::{error, warn};

pub type MessagePayload = Arc<[String]>;
pub type MessageBroadcast = Sender<MessagePayload>;

pub struct MessageBroadcastService {
  event_channel: EventChannel,
  message_broadcast: MessageBroadcast,
}

impl MessageBroadcastService {
  #[inline]
  pub const fn new(event_channel: EventChannel, message_broadcast: MessageBroadcast) -> Self {
    Self {
      event_channel,
      message_broadcast,
    }
  }
}

struct SerializeTask {
  broadcast: MessageBroadcast,
}

impl BackgroundService for MessageBroadcastService {
  fn run(
    &self,
    token: CancellationToken,
  ) -> std::pin::Pin<Box<dyn core::future::Future<Output = ()> + Send>> {
    let mut listener = self.event_channel.subscribe();
    let task = SerializeTask {
      broadcast: self.message_broadcast.clone(),
    };

    Box::pin(async move {
      'MAIN: loop {
        select! {
          event = listener.recv() => {
            match event {
              Ok(event) => {
                task.next(event).await;
              },
              Err(RecvError::Closed) => break 'MAIN,
              Err(RecvError::Lagged(lagged)) => {
                warn!(
                  message = "message serialization service can't keep up with system events",
                  lagged_event_count = lagged
                )
              }
            }
          }
          _ = token.cancelled() =>  {
              break 'MAIN;
          }
        }
      }
    })
  }
}

impl SerializeTask {
  async fn next(&self, event: Arc<SystemEvent>) {
    if self.broadcast.receiver_count() < 1 {
      //skip whole serialization if there are no active receiver (wss socket in this case)
      return;
    }

    let timestamp = Utc::now().timestamp_millis();
    let payload = match event.as_ref() {
      SystemEvent::DeviceAddition { devices, namespace } => {
        Self::process_device_addition(devices, namespace, timestamp)
      }
      SystemEvent::DeviceRemoval { devices, namespace } => {
        Self::process_device_removal(devices, namespace, timestamp)
      }
      SystemEvent::DeviceUpdate { devices, namespace } => {
        Self::process_device_update(devices, namespace, timestamp)
      }
      SystemEvent::DeviceStatusChange { changes, namespace } => {
        Self::process_device_status_update(changes, namespace, timestamp)
      }
      SystemEvent::DaemonStatusUpdate { status, namespace } => {
        Self::process_daemon_state(*status, namespace, timestamp)
      }
      SystemEvent::ClientConnection { devices, namespace } => {
        Self::process_client_connect(devices, namespace, timestamp)
      }
      SystemEvent::ClientDisconnection { devices, namespace } => {
        Self::process_client_disconnect(devices, namespace, timestamp)
      }
    };

    match payload {
      Ok(v) => {
        let payload = Arc::from(v);
        _ = self.broadcast.send(payload).inspect_err(|err| {
          warn!(
          message = "sending serialized message data to broadcast channel failed",
          reason = %err)
        });
      }
      Err(err) => {
        error!(message = "unable to serialize event", reason = %err);
      }
    };
  }

  fn process_device_addition(
    devices: &[UpsName],
    namespace: &str,
    timestamp: i64,
  ) -> Result<MessagePayload, serde_json::error::Error> {
    let mut data = Vec::with_capacity(devices.len());

    for name in devices {
      let message = NutEventMessage::DeviceConnected {
        name,
        namespace,
        timestamp,
      };

      let value = serde_json::to_string(&message)?;
      data.push(value);
    }

    Ok(Arc::from(data))
  }

  fn process_device_removal(
    devices: &[UpsName],
    namespace: &str,
    timestamp: i64,
  ) -> Result<MessagePayload, serde_json::error::Error> {
    let mut data = Vec::with_capacity(devices.len());

    for name in devices {
      let message = NutEventMessage::DeviceRemoved {
        name,
        namespace,
        timestamp,
      };

      let value = serde_json::to_string(&message)?;
      data.push(value);
    }

    Ok(Arc::from(data))
  }

  fn process_device_update(
    devices: &[UpsName],
    namespace: &str,
    timestamp: i64,
  ) -> Result<MessagePayload, serde_json::error::Error> {
    let mut data = Vec::with_capacity(devices.len());

    for name in devices {
      let message = NutEventMessage::DeviceUpdate {
        name,
        namespace,
        timestamp,
      };

      let value = serde_json::to_string(&message)?;
      data.push(value);
    }

    Ok(Arc::from(data))
  }

  fn process_device_status_update(
    changes: &[DeviceStatusChange],
    namespace: &str,
    timestamp: i64,
  ) -> Result<MessagePayload, serde_json::error::Error> {
    let mut data = Vec::with_capacity(changes.len());

    for change in changes {
      let message = NutEventMessage::DeviceStatus {
        status_new: change.status_new,
        status_old: change.status_old,
        events: UpsEvents::new(change.status_old, change.status_new),
        name: &change.name,
        namespace,
        timestamp,
      };

      let value = serde_json::to_string(&message)?;
      data.push(value);
    }

    Ok(Arc::from(data))
  }

  fn process_daemon_state(
    status: ConnectionStatus,
    namespace: &str,
    timestamp: i64,
  ) -> Result<MessagePayload, serde_json::error::Error> {
    let message = NutEventMessage::DaemonStatus {
      status,
      namespace,
      timestamp,
    };

    let value = serde_json::to_string(&message)?;
    Ok(Arc::from([value]))
  }
  fn process_client_connect(
    devices: &[DeviceClientInfo],
    namespace: &str,
    timestamp: i64,
  ) -> Result<MessagePayload, serde_json::error::Error> {
    let mut data = Vec::new();

    for client_info in devices {
      for client_ip in client_info.clients.iter() {
        let message = NutEventMessage::ClientConnect {
          client_ip: *client_ip,
          name: &client_info.name,
          namespace,
          timestamp,
        };

        let value = serde_json::to_string(&message)?;
        data.push(value);
      }
    }

    Ok(Arc::from(data))
  }

  fn process_client_disconnect(
    devices: &[DeviceClientInfo],
    namespace: &str,
    timestamp: i64,
  ) -> Result<MessagePayload, serde_json::error::Error> {
    let mut data = Vec::new();

    for client_info in devices {
      for client_ip in client_info.clients.iter() {
        let message = NutEventMessage::ClientDisconnect {
          client_ip: *client_ip,
          name: &client_info.name,
          namespace,
          timestamp,
        };

        let value = serde_json::to_string(&message)?;
        data.push(value);
      }
    }

    Ok(Arc::from(data))
  }
}
