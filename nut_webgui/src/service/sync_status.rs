use super::BackgroundService;
use crate::{
  diff_utils::Diff,
  event::{DeviceStatusChange, EventBatch, EventChannel, SystemEvent},
  state::ServerState,
};
use chrono::Utc;
use futures::future::join_all;
use nut_webgui_upsmc::{
  UpsName, VarName,
  clients::{AsyncNutClient, NutPoolClient},
  ups_status::UpsStatus,
};
use std::{sync::Arc, time::Duration};
use tokio::{
  join, select,
  sync::RwLock,
  time::{Instant, Interval, MissedTickBehavior, interval},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

pub struct StatusSyncService {
  client: NutPoolClient,
  event_channel: EventChannel,
  state: Arc<RwLock<ServerState>>,
  poll_freq: Duration,
  poll_interval: Duration,
}

impl StatusSyncService {
  pub fn new(
    client: NutPoolClient,
    event_channel: EventChannel,
    state: Arc<RwLock<ServerState>>,
    poll_interval: Duration,
    poll_freq: Duration,
  ) -> Self {
    Self {
      client,
      state,
      event_channel,
      poll_interval,
      poll_freq,
    }
  }
}

impl BackgroundService for StatusSyncService {
  fn run(
    &self,
    token: CancellationToken,
  ) -> std::pin::Pin<Box<dyn core::future::Future<Output = ()> + Send>> {
    let client = self.client.clone();
    let event_channel = self.event_channel.clone();
    let state = self.state.clone();
    let poll_freq = self.poll_freq;
    let poll_interval = {
      if self.poll_interval >= self.poll_freq {
        self.poll_freq
      } else {
        self.poll_interval
      }
    };

    Box::pin(async move {
      let task = StatusSyncTask {
        client,
        event_channel,
        state,
      };

      let mut interval = UpsPollInterval::new(poll_interval, poll_freq);

      'MAIN: loop {
        let poll_type = select! {
          poll_type = interval.tick() => {
            debug!(message = "starting device status sync", poll_type=%poll_type);
            poll_type
          }
          _ = token.cancelled() => { break 'MAIN; }
        };

        match poll_type {
          UpsPollType::Full => {
            select! {
              _ = task.state_sync() => {
                debug!(message = "full device status sync completed");
              }
              _ = token.cancelled() => { break 'MAIN; }
            };
          }
          UpsPollType::Partial => {
            select! {
              _ = task.status_sync() => {
                debug!(message = "partial device status sync completed");
              }
              _ = token.cancelled() => { break 'MAIN; }
            };
          }
        }
      }

      debug!(message = "device status sync stopped");
    })
  }
}

struct StatusSyncTask {
  client: NutPoolClient,
  state: Arc<RwLock<ServerState>>,
  event_channel: EventChannel,
}

impl StatusSyncTask {
  async fn snapshot_device_names(&self) -> Vec<UpsName> {
    let read_lock = self.state.read().await;
    read_lock.devices.keys().cloned().collect()
  }

  /// Only syncs `ups.status` variables for existing devices.
  pub async fn status_sync(&self) {
    let devices = self.snapshot_device_names().await;

    if devices.is_empty() {
      debug!(message = "no device available, nothing to sync");
      return;
    }

    let responses = join_all(devices.iter().map(|device| async move {
      let result = self.client.get_var(device, VarName::UPS_STATUS).await;
      (device, result)
    }))
    .await;

    let mut changes: Vec<DeviceStatusChange> = Vec::with_capacity(responses.len());

    {
      let mut write_lock = self.state.write().await;

      for (device, result) in responses {
        match result {
          Ok(variable) => {
            if let Some(entry) = write_lock.devices.get_mut(&variable.ups_name) {
              let old_status = entry.status;
              let new_status = UpsStatus::from(&variable.value);

              entry.status = new_status;
              entry.variables.insert(variable.name, variable.value);
              entry.last_modified = Utc::now();

              if old_status != new_status {
                changes.push(DeviceStatusChange {
                  new_status,
                  old_status,
                  name: variable.ups_name,
                });
              }
            }
          }
          Err(err) => {
            debug!(message = "failed to read ups status", device = %device, reason = %err)
          }
        }
      }
    }

    if !changes.is_empty() {
      let send_result = self
        .event_channel
        .send(SystemEvent::DeviceStatusChange { changes });

      if let Err(err) = send_result {
        warn!(message = "cannot write new system events to channel", reason = %err);
      }
    }
  }

  pub async fn state_sync(&self) {
    let devices = self.snapshot_device_names().await;

    if devices.is_empty() {
      debug!(message = "no device available, nothing to sync");
      return;
    }

    let responses = join_all(devices.iter().map(|device| async move {
      let (variables, clients, commands) = join!(
        self.client.list_var(device),
        self.client.list_client(device),
        self.client.list_cmd(device)
      );

      (device, variables, clients, commands)
    }))
    .await;

    let mut events = EventBatch::new();

    {
      let mut write_lock = self.state.write().await;

      for result in responses {
        match result {
          (device, Ok(var_list), Ok(clients), Ok(commands)) => {
            if let Some(entry) = write_lock.devices.get_mut(device) {
              if let Some(status_value) = var_list.variables.get(VarName::UPS_STATUS) {
                let new_status = UpsStatus::from(status_value);
                let old_status = entry.status;

                if old_status != new_status {
                  entry.status = new_status;

                  events.status_change(var_list.ups_name, old_status, new_status);
                }
              }

              let client_diff = entry.attached.as_slice().into_diff(&clients.ips);

              if !client_diff.connected.is_empty() {
                for client in client_diff.connected.iter() {
                  info!(message = "new client attached to ups", device = %device, client = %client)
                }

                events.client_connection(device.clone(), client_diff.connected);
              }

              if !client_diff.disconnected.is_empty() {
                for client in client_diff.disconnected.iter() {
                  info!(message = "client detached from ups", device = %device, client = %client)
                }

                events.client_disconnect(device.clone(), client_diff.disconnected);
              }

              entry.variables = var_list.variables;
              entry.attached = clients.ips;
              entry.commands = commands.cmds;
              entry.last_modified = Utc::now();

              events.updated_device(clients.ups_name);
            }
          }
          (device, vars_results, clients_result, cmds_result) => {
            if let Err(err) = vars_results {
              debug!(message = "failed to read ups variables", device = %device, reason = %err)
            }
            if let Err(err) = clients_result {
              debug!(message = "failed to read ups attached clients", device = %device, reason = %err)
            }
            if let Err(err) = cmds_result {
              debug!(message = "failed to read ups commands", device = %device, reason = %err)
            }
          }
        }
      }
    }

    if let Err(err) = events.send(&self.event_channel) {
      warn!(message = "cannot write new system events to channel", reason = %err);
    }
  }
}

struct UpsPollInterval {
  interval: Interval,
  last_full_sync: Option<Instant>,
  full_sync_period: Duration,
}

#[derive(Debug, Clone, Copy)]
enum UpsPollType {
  Full,
  Partial,
}

impl std::fmt::Display for UpsPollType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      UpsPollType::Full => f.write_str("Full"),
      UpsPollType::Partial => f.write_str("Partial"),
    }
  }
}

impl UpsPollInterval {
  pub fn new(poll_interval: Duration, poll_freq: Duration) -> Self {
    let mut interval = interval(poll_interval);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    Self {
      interval,
      last_full_sync: None,
      full_sync_period: poll_freq,
    }
  }

  #[inline]
  pub async fn tick(&mut self) -> UpsPollType {
    let instant = self.interval.tick().await;

    match self.last_full_sync {
      Some(last_sync) => {
        if instant.duration_since(last_sync) >= self.full_sync_period {
          self.last_full_sync = Some(instant);
          UpsPollType::Full
        } else {
          UpsPollType::Partial
        }
      }
      None => {
        self.last_full_sync = Some(instant);
        UpsPollType::Full
      }
    }
  }
}
