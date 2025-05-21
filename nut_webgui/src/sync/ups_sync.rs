use crate::{
  event::{ChannelClosedError, EventChannel, SystemEvent},
  state::{DaemonStatus, DeviceEntry, ServerState},
  sync::error::IntoLoadError as _,
};
use chrono::Utc;
use nut_webgui_upsmc::{
  UpsName, VarName,
  clients::{AsyncNutClient, NutPoolClient},
  responses::UpsDevice,
  ups_status::UpsStatus,
};
use std::{collections::HashMap, str::FromStr, sync::Arc, time::Duration};
use tokio::{join, net::ToSocketAddrs, select, sync::RwLock, task::JoinSet, time::interval};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use super::error::{DeviceLoadError, SyncTaskError};

/// Synchronizes device list from UPSD.
pub struct DeviceSyncService<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  task: SyncTask<A>,
  cancellation: CancellationToken,
  poll_interval: Duration,
}

impl<A> DeviceSyncService<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  pub fn new(
    client: NutPoolClient<A>,
    event_channel: EventChannel,
    state: Arc<RwLock<ServerState>>,
    poll_interval: Duration,
    cancellation: CancellationToken,
  ) -> Self {
    Self {
      task: SyncTask {
        client,
        state,
        event_channel,
      },
      cancellation,
      poll_interval,
    }
  }

  pub async fn run(self) {
    let Self {
      cancellation,
      task,
      poll_interval: interval_duration,
      ..
    } = self;

    let mut interval = interval(interval_duration);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    'MAIN: loop {
      select! {
        _ = interval.tick() => { }
        _ = cancellation.cancelled() =>  {
            break 'MAIN;
        }
      };

      select! {
        v = task.next() => {
          if let Err(err) = v {
            error!(message = "sync failed: unable to communicate with upsd", reason=%err)
          }
        }
        _ = cancellation.cancelled() =>  {
            break 'MAIN;
        }
      };
    }

    info!(message = "ups sync task finished");
  }
}

struct SyncTask<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  client: NutPoolClient<A>,
  state: Arc<RwLock<ServerState>>,
  event_channel: EventChannel,
}

impl<A> SyncTask<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  pub async fn next(&self) -> Result<(), SyncTaskError> {
    let remote_devices = match self.client.list_ups().await {
      Ok(res) => Ok(res),
      Err(err) => {
        let mut write_lock = self.state.write().await;
        write_lock.state.status = DaemonStatus::Dead;
        write_lock.devices = HashMap::new();

        Err(err)
      }
    }?;

    let local_devices: HashMap<_, _> = {
      let state_lock = self.state.read().await;
      state_lock
        .devices
        .iter()
        .map(|(k, v)| {
          (
            k.clone(),
            UpsDevice {
              ups_name: v.name.clone(),
              desc: v.desc.clone(),
            },
          )
        })
        .collect()
    };

    let total_device_count = remote_devices.devices.len();
    let diff = DeviceDiff::new(local_devices, remote_devices.devices);

    let mut failure_count = 0;
    let mut task_set = JoinSet::new();

    for device in diff.new.into_iter() {
      let client = self.client.clone();
      task_set.spawn(load_device_entry(client, device));
    }

    let mut new_devices: Vec<DeviceEntry> = Vec::with_capacity(task_set.len());

    while let Some(result) = task_set.join_next().await {
      match result {
        Ok(Ok(device)) => new_devices.push(device),
        Ok(Err(err)) => {
          failure_count += 1;
          error!(message = "unable to get device details from nut upsd", reason = %err.inner, ups_name = %err.name)
        }
        Err(err) => {
          failure_count += 1;
          error!(message = "cannot join device load task", reason = %err)
        }
      }
    }

    if failure_count >= total_device_count {
      let mut write_lock = self.state.write().await;

      if write_lock.state.status != DaemonStatus::Dead {
        write_lock.state.status = DaemonStatus::Dead;

        if let Err(err) = self.event_channel.send(SystemEvent::UpsdStatus {
          status: DaemonStatus::Dead,
        }) {
          warn!(message = "Unable to send status event", reason= %err);
        }
      }

      write_lock.devices = HashMap::new();
      write_lock.state.last_ups_sync = Some(Utc::now());

      Err(SyncTaskError::DeviceLoadFailed)
    } else {
      let mut events = EventBatch::new();
      let mut write_lock = self.state.write().await;

      for entry in new_devices.into_iter() {
        info!(message = "new device found", device = %&entry.name);

        events.push_new_device(entry.name.clone());
        write_lock.devices.insert(entry.name.clone(), entry);
      }

      for entry in diff.updated.into_iter() {
        if let Some(device) = write_lock.devices.get_mut(&entry.ups_name) {
          info!(message = "device description updated", device = %&device.name);

          events.push_updated_device(entry.ups_name);
          device.desc = entry.desc;
        }
      }

      for device_name in diff.removed.into_iter() {
        info!(message = "device disconnected", device = %&device_name);

        _ = write_lock.devices.remove(&device_name);
        events.push_removed_device(device_name);
      }

      if write_lock.state.status != DaemonStatus::Online {
        write_lock.state.status = DaemonStatus::Online;
        events.set_upsd_status(DaemonStatus::Online);
      }

      write_lock.state.last_ups_sync = Some(Utc::now());

      if let Err(err) = events.send(&self.event_channel) {
        warn!(message = "Unable to send events", reason= %err);
      }

      Ok(())
    }
  }
}

struct EventBatch {
  new: Vec<UpsName>,
  removed: Vec<UpsName>,
  updated: Vec<UpsName>,
  upsd_status: Option<DaemonStatus>,
}

impl EventBatch {
  pub fn new() -> Self {
    Self {
      upsd_status: None,
      new: Vec::new(),
      updated: Vec::new(),
      removed: Vec::new(),
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

  pub fn set_upsd_status(&mut self, status: DaemonStatus) {
    self.upsd_status = Some(status);
  }

  pub fn send(self, channel: &EventChannel) -> Result<(), ChannelClosedError> {
    if !self.new.is_empty() {
      _ = channel.send(SystemEvent::DevicesAdded { devices: self.new })?;
    }

    if !self.removed.is_empty() {
      _ = channel.send(SystemEvent::DevicesRemoved {
        devices: self.removed,
      })?;
    }

    if !self.updated.is_empty() {
      _ = channel.send(SystemEvent::DevicesUpdated {
        devices: self.updated,
      })?;
    }

    if let Some(status) = self.upsd_status {
      _ = channel.send(SystemEvent::UpsdStatus { status })?;
    }

    Ok(())
  }
}

struct DeviceDiff {
  new: Vec<UpsDevice>,
  removed: Vec<UpsName>,
  updated: Vec<UpsDevice>,
}

impl DeviceDiff {
  pub fn new<I>(mut local_devices: HashMap<UpsName, UpsDevice>, remote_devices: I) -> Self
  where
    I: IntoIterator<Item = UpsDevice>,
  {
    let mut result = DeviceDiff {
      new: Vec::new(),
      updated: Vec::new(),
      removed: Vec::new(),
    };

    for device in remote_devices.into_iter() {
      match local_devices.remove_entry(&device.ups_name) {
        Some((_, local_device)) => {
          if local_device.desc != device.desc {
            result.updated.push(device);
          }
        }
        None => {
          result.new.push(device);
        }
      }
    }

    if local_devices.len() > 0 {
      result.removed = local_devices.into_iter().map(|(_, v)| v.ups_name).collect();
    }

    result
  }
}

async fn load_device_entry<A>(
  client: NutPoolClient<A>,
  device: UpsDevice,
) -> Result<DeviceEntry, DeviceLoadError>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  let UpsDevice { ups_name, desc } = device;

  let (vars, rw_vars, commands, clients) = join!(
    client.list_var(&ups_name),
    client.list_rw(&ups_name),
    client.list_cmd(&ups_name),
    client.list_client(&ups_name),
  );

  let variables = vars.map_load_err(&ups_name)?.variables;
  let status = match variables.get(&VarName::UPS_STATUS) {
    Some(nut_webgui_upsmc::Value::String(inner)) => UpsStatus::from_str(inner).unwrap_or_default(),
    _ => UpsStatus::default(),
  };
  let attached = clients.map_load_err(&ups_name)?.ips;
  let commands = commands.map_load_err(&ups_name)?.cmds;
  let rw_variables = rw_vars
    .map_load_err(&ups_name)?
    .variables
    .iter()
    .map(|(k, _)| k.clone())
    .collect(); // FIX: RW need it's own type. It can be enum, range etc

  let entry = DeviceEntry {
    name: ups_name,
    desc,
    variables,
    attached,
    commands,
    rw_variables,
    status,
    last_modified: Utc::now(),
  };

  Ok(entry)
}
