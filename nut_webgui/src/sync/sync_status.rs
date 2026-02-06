use crate::{
  background_service::BackgroundService,
  event::{DeviceStatusChange, SystemEvent, batch::EventBatch, channel::EventChannel},
  state::{ClientInfo, UpsdState},
  sync::reverse_dns::lookup_ip,
};
use chrono::Utc;
use futures::future::join_all;
use nut_webgui_upsmc::{UpsName, VarName, client::AsyncNutClient, ups_status::UpsStatus};
use std::{net::IpAddr, sync::Arc, time::Duration};
use tokio::{
  select,
  time::{Instant, Interval, MissedTickBehavior, interval},
  try_join,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

pub struct StatusSyncService {
  event_channel: EventChannel,
  state: Arc<UpsdState>,
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

struct StatusSyncTask {
  state: Arc<UpsdState>,
  event_channel: EventChannel,
}

struct ClientDiff {
  pub connected: Vec<IpAddr>,
  pub disconnected: Vec<IpAddr>,
}

impl StatusSyncService {
  pub fn new(event_channel: EventChannel, state: Arc<UpsdState>) -> Self {
    Self {
      state,
      event_channel,
    }
  }
}

impl BackgroundService for StatusSyncService {
  fn run(
    &self,
    token: CancellationToken,
  ) -> std::pin::Pin<Box<dyn core::future::Future<Output = ()> + Send>> {
    let event_channel = self.event_channel.clone();
    let state = self.state.clone();

    Box::pin(async move {
      let namespace = state.namespace.clone();
      let poll_freq = Duration::from_secs(state.config.poll_freq);
      let poll_interval = {
        if state.config.poll_interval >= state.config.poll_freq {
          Duration::from_secs(state.config.poll_freq)
        } else {
          Duration::from_secs(state.config.poll_interval)
        }
      };

      let task = StatusSyncTask {
        event_channel,
        state,
      };

      let mut interval = UpsPollInterval::new(poll_interval, poll_freq);

      'MAIN: loop {
        let poll_type = select! {
          poll_type = interval.tick() => poll_type,
          _ = token.cancelled() => { break 'MAIN; }
        };

        match poll_type {
          UpsPollType::Full => {
            select! {
              _ = task.state_sync() => {
                debug!(
                  message = "full device status sync completed",
                  namespace = %namespace
                );
              }
              _ = token.cancelled() => { break 'MAIN; }
            };
          }
          UpsPollType::Partial => {
            select! {
              _ = task.status_sync() => {
                debug!(
                  message = "partial device status sync completed",
                  namespace = %namespace
                );
              }
              _ = token.cancelled() => { break 'MAIN; }
            };
          }
        }
      }

      debug!(
        message = "device status sync stopped",
        namespace = %namespace
      );
    })
  }
}

impl StatusSyncTask {
  async fn snapshot_active_device_names(&self) -> Vec<UpsName> {
    let read_lock = self.state.daemon_state.read().await;
    read_lock
      .devices
      .iter()
      .filter_map(|(k, v)| {
        if v.status.has(UpsStatus::NOCOMM) {
          None
        } else {
          Some(k.clone())
        }
      })
      .collect()
  }

  /// Only syncs `ups.status` variables for existing devices.
  pub async fn status_sync(&self) {
    let devices = self.snapshot_active_device_names().await;

    if devices.is_empty() {
      debug!(
        message = "no device available, nothing to sync",
        namespace = %self.state.namespace
      );
      return;
    }

    let responses = join_all(devices.iter().map(|device| async move {
      self
        .state
        .connection_pool
        .get_var(device, VarName::UPS_STATUS)
        .await
        .map_err(|err| (device, err))
    }))
    .await;

    let mut changes: Vec<DeviceStatusChange> = Vec::with_capacity(responses.len());

    {
      let mut write_lock = self.state.daemon_state.write().await;

      for result in responses {
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
                  status_new: new_status,
                  status_old: old_status,
                  name: variable.ups_name,
                });
              }
            }
          }
          Err((device, err)) => {
            debug!(
              message = "failed to read ups status",
              namespace = %self.state.namespace,
              device = %device, reason = %err
            );
          }
        }
      }
    };

    if !changes.is_empty() {
      _ = self
        .event_channel
        .send(SystemEvent::DeviceStatusChange {
          changes,
          namespace: self.state.namespace.clone(),
        })
        .inspect_err(|err| {
          warn!(
            message = "cannot write new system events to channel",
            namespace = %self.state.namespace,
            reason = %err
          );
        });
    }
  }

  pub async fn state_sync(&self) {
    let devices = self.snapshot_active_device_names().await;

    if devices.is_empty() {
      debug!(
        message = "no device available, nothing to sync",
        namespace = %self.state.namespace
      );

      return;
    }

    let client = &self.state.connection_pool;
    let responses = join_all(devices.into_iter().map(|device| async move {
      try_join!(
        client.list_var(&device),
        client.list_client(&device),
        client.list_cmd(&device)
      )
      .map_err(|err| (device.clone(), err))
      .map(|(var, client, cmd)| (device, var, client, cmd))
    }))
    .await;

    let mut events = EventBatch::new(self.state.namespace.clone());

    {
      let mut write_lock = self.state.daemon_state.write().await;

      for result in responses {
        match result {
          Ok((device_name, var_list, clients, commands)) => {
            if let Some(entry) = write_lock.devices.get_mut(&device_name) {
              if let Some(status_value) = var_list.variables.get(VarName::UPS_STATUS) {
                let new_status = UpsStatus::from(status_value);
                let old_status = entry.status;

                if old_status != new_status {
                  entry.status = new_status;
                  events.status_change(device_name.clone(), old_status, new_status);
                }
              }

              let client_diff = ClientDiff::diff(&entry.attached, &clients.ips);

              if !client_diff.disconnected.is_empty() {
                for client_ip in client_diff.disconnected.iter() {
                  if let Some(idx) = entry.attached.iter().position(|c| c.addr == *client_ip) {
                    _ = entry.attached.swap_remove(idx);

                    info!(
                      message = "client detached from ups",
                      namespace = %self.state.namespace,
                      device = %device_name,
                      client = %client_ip
                    );
                  }
                }

                events.client_disconnect(device_name.clone(), client_diff.disconnected);
              }

              if !client_diff.connected.is_empty() {
                for client_ip in client_diff.connected.iter() {
                  let client_name = if !client_ip.is_loopback() {
                    lookup_ip(*client_ip).map_or(None, |v| Some(v))
                  } else {
                    None
                  };

                  entry.attached.push(ClientInfo {
                    addr: *client_ip,
                    name: client_name,
                  });

                  info!(
                    message = "new client attached to ups",
                    namespace = %self.state.namespace,
                    device = %device_name,
                    client = %client_ip
                  );
                }

                events.client_connection(device_name.clone(), client_diff.connected);
              }

              entry.variables = var_list.variables;
              entry.commands = commands.cmds;
              entry.last_modified = Utc::now();
              events.updated_device(device_name);
            }
          }
          Err((device_name, err)) => {
            error!(
              message = "failed to read ups details",
              namespace = %self.state.namespace,
              device = %device_name,
              reason = %err
            );

            if let Some(entry) = write_lock.devices.get_mut(&device_name) {
              events.status_change(device_name, entry.status, UpsStatus::NOCOMM);
              entry.mark_as_dead_with(UpsStatus::NOCOMM);
            }
          }
        }
      }
    };

    _ = self.event_channel.send_batch(events).inspect_err(|err| {
      warn!(
        message = "cannot write new system events to channel",
        namespace = %self.state.namespace,
        reason = %err
      );
    });
  }
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

impl ClientDiff {
  pub fn diff(source: &[ClientInfo], target: &[IpAddr]) -> Self {
    let mut diff = ClientDiff {
      connected: Vec::new(),
      disconnected: Vec::new(),
    };

    for old in source.iter() {
      if target.iter().find(|v| **v == old.addr).is_none() {
        diff.disconnected.push(old.addr);
      }
    }

    for new in target.iter() {
      if source.iter().find(|v| v.addr == *new).is_none() {
        diff.connected.push(*new);
      }
    }

    diff
  }
}
