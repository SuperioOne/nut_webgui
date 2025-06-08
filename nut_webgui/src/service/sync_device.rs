use super::{
  BackgroundService,
  error::{DeviceLoadError, IntoLoadError, SyncTaskError},
};
use crate::{
  device_entry::{DeviceEntry, VarDetail},
  event::{EventBatch, EventChannel, SystemEvent},
  state::{DaemonStatus, ServerState},
};
use chrono::Utc;
use futures::future::join_all;
use nut_webgui_upsmc::{
  UpsName, Value, VarName, VarType,
  clients::{AsyncNutClient, NutPoolClient},
  responses::UpsDevice,
  ups_status::UpsStatus,
};
use std::{collections::HashMap, net::ToSocketAddrs, sync::Arc, time::Duration};
use tokio::{join, select, sync::RwLock, task::JoinSet, time::interval, try_join};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

/// Synchronizes device list from UPSD.
pub struct DeviceSyncService<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  client: NutPoolClient<A>,
  event_channel: EventChannel,
  poll_interval: Duration,
  state: Arc<RwLock<ServerState>>,
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
  ) -> Self {
    Self {
      client,
      event_channel,
      poll_interval,
      state,
    }
  }
}

impl<A> BackgroundService for DeviceSyncService<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  fn run(
    &self,
    token: CancellationToken,
  ) -> core::pin::Pin<Box<dyn core::future::Future<Output = ()> + Send + Sync + 'static>> {
    let client = self.client.clone();
    let event_channel = self.event_channel.clone();
    let poll_interval = self.poll_interval;
    let state = self.state.clone();

    Box::pin(async move {
      let task = DeviceSyncTask {
        state,
        client,
        event_channel,
      };
      let mut interval = interval(poll_interval);
      interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

      'MAIN: loop {
        select! {
          _ = interval.tick() => debug!(message = "starting remote device sync"),
          _ = token.cancelled() =>  { break 'MAIN; }
        };

        select! {
          v = task.next() => {
            match v {
              Ok(_) => debug!(message = "remote device sync completed") ,
              Err(err) => error!(message = "remote device sync failed", reason=%err)
            }
          }
          _ = token.cancelled() =>  { break 'MAIN; }
        };
      }

      debug!(message = "device sync task stopped");
    })
  }
}

struct DeviceSyncTask<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  client: NutPoolClient<A>,
  state: Arc<RwLock<ServerState>>,
  event_channel: EventChannel,
}

impl<A> DeviceSyncTask<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  pub async fn next(&self) -> Result<(), SyncTaskError> {
    let remote_details = try_join!(
      self.client.list_ups(),
      self.client.get_protver(),
      self.client.get_ver(),
    );

    let (remote, prot_ver, ver) = match remote_details {
      Ok(res) => Ok(res),
      Err(err) => {
        let mut write_lock = self.state.write().await;

        if write_lock.remote_state.status != DaemonStatus::Dead {
          write_lock.remote_state.status = DaemonStatus::Dead;
          write_lock.remote_state.prot_ver = None;
          write_lock.remote_state.ver = None;

          error!(message = "ups daemon is disconnected", reason = %err);

          _ = self.event_channel.send(SystemEvent::UpsdStatus {
            status: DaemonStatus::Dead,
          });
        }

        write_lock.devices.clear();

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

    let total_device_count = remote.devices.len();
    let diff = DeviceDiff::new(local_devices, remote.devices);

    let mut failure_count = 0;
    let mut task_set = JoinSet::new();

    for device in diff.new.into_iter() {
      let client = self.client.clone();
      task_set.spawn(Self::load_device_entry(client, device));
    }

    let mut new_devices: Vec<DeviceEntry> = Vec::new();

    while let Some(result) = task_set.join_next().await {
      match result {
        Ok(Ok(device)) => new_devices.push(device),
        Ok(Err(err)) => {
          failure_count += 1;
          error!(message = "unable to get device details from nut upsd", reason = %err.inner, device = %err.name)
        }
        Err(err) => {
          failure_count += 1;
          error!(message = "cannot join device load task", reason = %err)
        }
      }
    }

    let mut write_lock = self.state.write().await;

    if failure_count >= total_device_count {
      if write_lock.remote_state.status != DaemonStatus::Dead {
        error!(
          message = "ups daemon is disconnected",
          reason = "received device list but unable to load device details"
        );

        write_lock.remote_state.status = DaemonStatus::Dead;
        write_lock.remote_state.prot_ver = None;
        write_lock.remote_state.ver = None;

        if let Err(err) = self.event_channel.send(SystemEvent::UpsdStatus {
          status: DaemonStatus::Dead,
        }) {
          warn!(message = "unable to send status event", reason= %err);
        }
      }

      write_lock.devices.clear();
      write_lock.remote_state.last_device_sync = Some(Utc::now());

      Err(SyncTaskError::DeviceLoadFailed)
    } else {
      let mut events = EventBatch::new();

      for entry in new_devices.into_iter() {
        info!(message = "device connected", device = %&entry.name);

        events.push_new_device(entry.name.clone());
        write_lock.devices.insert(entry.name.clone(), entry);
      }

      for entry in diff.updated.into_iter() {
        if let Some(device) = write_lock.devices.get_mut(&entry.ups_name) {
          info!(message = "device details updated", device = %&device.name);

          events.push_updated_device(entry.ups_name);
          device.desc = entry.desc;
        }
      }

      for device_name in diff.removed.into_iter() {
        info!(message = "device disconnected", device=%device_name);

        _ = write_lock.devices.remove(&device_name);
        events.push_removed_device(device_name);
      }

      if write_lock.remote_state.status != DaemonStatus::Online {
        info!(message = "ups daemon is online");

        write_lock.remote_state.status = DaemonStatus::Online;
        events.set_upsd_status(DaemonStatus::Online);
      }

      write_lock.remote_state.last_device_sync = Some(Utc::now());
      write_lock.remote_state.prot_ver = Some(prot_ver.value.into_boxed_str());
      write_lock.remote_state.ver = Some(ver.value.into_boxed_str());

      if let Err(err) = events.send(&self.event_channel) {
        warn!(message = "unable to send events", reason= %err);
      }

      Ok(())
    }
  }

  async fn load_device_entry(
    client: NutPoolClient<A>,
    device: UpsDevice,
  ) -> Result<DeviceEntry, DeviceLoadError> {
    let UpsDevice { ups_name, desc } = device;

    let (clients, commands, rw_vars, vars) = join!(
      client.list_client(&ups_name),
      client.list_cmd(&ups_name),
      client.list_rw(&ups_name),
      client.list_var(&ups_name),
    );

    let variables = vars.map_load_err(&ups_name)?.variables;
    let status = match variables.get(VarName::UPS_STATUS) {
      Some(value) => UpsStatus::from(value),
      _ => UpsStatus::default(),
    };
    let attached = clients.map_load_err(&ups_name)?.ips;
    let commands = commands.map_load_err(&ups_name)?.cmds;

    let rw_vars = join_all(
      rw_vars
        .map_load_err(&ups_name)?
        .variables
        .into_iter()
        .map(|(var_name, _)| Self::load_var_detail(client.clone(), &ups_name, var_name)),
    )
    .await;

    let mut rw_variables = HashMap::with_capacity(rw_vars.len());

    for result in rw_vars {
      match result {
        Ok((var_name, detail)) => {
          _ = rw_variables.insert(var_name, detail);
        }
        Err(err) => {
          warn!(message = "failed to get RW variable type details, variable will be displayed as read-only", device = %err.name, reason = %err.inner );
        }
      };
    }

    let entry = DeviceEntry {
      attached,
      commands,
      desc,
      last_modified: Utc::now(),
      name: ups_name,
      rw_variables,
      status,
      variables,
    };

    Ok(entry)
  }

  async fn load_var_detail(
    client: NutPoolClient<A>,
    ups_name: &UpsName,
    var_name: VarName,
  ) -> Result<(VarName, VarDetail), DeviceLoadError> {
    let type_info = client
      .get_var_type(ups_name, &var_name)
      .await
      .map_load_err(ups_name)?;

    for var_type in type_info.var_types {
      match var_type {
        VarType::ReadWrite => continue,
        VarType::Enum => {
          let enum_list = client
            .list_enum(ups_name, &var_name)
            .await
            .map_load_err(ups_name)?;

          if enum_list.values.is_empty() {
            warn!(message = "nut driver reports variable type as enum, but it does not provide any enum option", var_name = %var_name, device = %ups_name);
          }

          return Ok((
            enum_list.name,
            VarDetail::Enum {
              options: enum_list.values,
            },
          ));
        }
        VarType::Range => {
          let mut range_list = client
            .list_range(ups_name, &var_name)
            .await
            .map_load_err(ups_name)?;

          return match range_list.ranges.pop() {
            Some((min, max)) => Ok((range_list.name, VarDetail::Range { min, max })),
            None => {
              warn!(message = "nut driver reports variable type as range, but it does not provide any range information", var_name = %var_name, device = %ups_name);
              Ok((
                range_list.name,
                VarDetail::Range {
                  min: Value::from(i64::MIN),
                  max: Value::from(i64::MAX),
                },
              ))
            }
          };
        }
        VarType::String { max_len } => {
          return Ok((var_name, VarDetail::String { max_len }));
        }
        VarType::Number => {
          return Ok((var_name, VarDetail::Number));
        }
      }
    }

    Ok((var_name, VarDetail::String { max_len: 64 }))
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
