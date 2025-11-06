use crate::{
  device_entry::{DeviceEntry, VarDetail},
  diff_utils::Diff,
  event::{EventBatch, EventChannel, SystemEvent},
  service::{
    BackgroundService,
    error::{DeviceLoadError, IntoLoadError, SyncTaskError},
  },
  state::{ConnectionStatus, UpsdState},
};
use chrono::Utc;
use futures::future::join_all;
use nut_webgui_upsmc::{
  UpsName, Value, VarName, VarType,
  client::{AsyncNutClient, NutPoolClient},
  response::UpsDevice,
  ups_status::UpsStatus,
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{join, select, task::JoinSet, time::interval, try_join};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

/// Synchronizes device list from UPSD.
pub struct DeviceSyncService {
  event_channel: EventChannel,
  state: Arc<UpsdState>,
}

impl DeviceSyncService {
  pub fn new(event_channel: EventChannel, state: Arc<UpsdState>) -> Self {
    Self {
      event_channel,
      state,
    }
  }
}

impl BackgroundService for DeviceSyncService {
  fn run(
    &self,
    token: CancellationToken,
  ) -> core::pin::Pin<Box<dyn core::future::Future<Output = ()> + Send>> {
    let event_channel = self.event_channel.clone();
    let state = self.state.clone();

    Box::pin(async move {
      let namespace = state.namespace.clone();
      let mut interval = interval(Duration::from_secs(state.config.poll_freq));
      interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

      let task = DeviceSyncTask {
        state,
        event_channel,
      };

      'MAIN: loop {
        select! {
          _ = interval.tick() => { },
          _ = token.cancelled() =>  { break 'MAIN; }
        };

        select! {
          v = task.next() => {
            match v {
              Ok(_) => {
                debug!(
                  message = "ups device sync completed",
                  namespace = %namespace
                );
              },
              Err(err) => {
                error!(
                  message = "ups device sync failed",
                  namespace = %namespace,
                  reason = %err
                );
              }
            }
          }
          _ = token.cancelled() =>  { break 'MAIN; }
        };
      }

      debug!(
        message = "device sync task stopped",
        namespace = %namespace,
      );
    })
  }
}

struct DeviceSyncTask {
  state: Arc<UpsdState>,
  event_channel: EventChannel,
}

impl DeviceSyncTask {
  pub async fn next(&self) -> Result<(), SyncTaskError> {
    let client = &self.state.connection_pool;
    let remote_details = try_join!(client.list_ups(), client.get_protver(), client.get_ver(),);

    let (remote, prot_ver, ver) = match remote_details {
      Ok(res) => Ok(res),
      Err(err) => {
        let mut write_lock = self.state.daemon_state.write().await;

        if write_lock.status != ConnectionStatus::Dead {
          write_lock.status = ConnectionStatus::Dead;
          write_lock.prot_ver = None;
          write_lock.ver = None;

          error!(
            message = "upsd is disconnected",
            namespace = %self.state.namespace,
            reason = %err
          );

          _ = self.event_channel.send(SystemEvent::DaemonStatusUpdate {
            status: ConnectionStatus::Dead,
            namespace: Box::from(self.state.namespace.as_ref()),
          });
        }

        write_lock.devices.clear();

        Err(err)
      }
    }?;

    let local_devices: HashMap<_, _> = {
      let state_lock = self.state.daemon_state.read().await;
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
    let diff = local_devices.into_diff(remote.devices);

    let mut failure_count = 0;
    let mut task_set = JoinSet::new();

    for device in diff.new.into_iter() {
      let client = self.state.connection_pool.clone();
      task_set.spawn(Self::load_device_entry(client, device));
    }

    let mut new_devices: Vec<DeviceEntry> = Vec::new();

    while let Some(result) = task_set.join_next().await {
      match result {
        Ok(Ok(device)) => new_devices.push(device),
        Ok(Err(err)) => {
          failure_count += 1;
          error!(
            message = "unable to get device details from upsd",
            namespace = %self.state.namespace,
            device = %err.name,
            reason = %err.inner
          );
        }
        Err(err) => {
          failure_count += 1;
          error!(
            message = "cannot join device load task",
            namespace = %self.state.namespace,
            reason = %err
          )
        }
      }
    }

    let mut write_lock = self.state.daemon_state.write().await;

    if failure_count >= total_device_count {
      if write_lock.status != ConnectionStatus::Dead {
        error!(
          message = "ups daemon is disconnected",
          namespace = %self.state.namespace,
          reason = "received device list but unable to load device details"
        );

        write_lock.status = ConnectionStatus::Dead;
        write_lock.prot_ver = None;
        write_lock.ver = None;

        if let Err(err) = self.event_channel.send(SystemEvent::DaemonStatusUpdate {
          status: ConnectionStatus::Dead,
          namespace: Box::from(self.state.namespace.as_ref()),
        }) {
          warn!(
            message = "unable to send status event",
            namespace = %self.state.namespace,
            reason = %err
          );
        }
      }

      write_lock.devices.clear();
      write_lock.last_device_sync = Some(Utc::now());

      Err(SyncTaskError::DeviceLoadFailed)
    } else {
      let mut events = EventBatch::new(self.state.namespace.as_ref());

      for entry in new_devices.into_iter() {
        info!(
          message = "device connected",
          namespace = %self.state.namespace,
          device = %entry.name
        );

        events.new_device(entry.name.clone());
        write_lock.devices.insert(entry.name.clone(), entry);
      }

      for entry in diff.updated.into_iter() {
        if let Some(device) = write_lock.devices.get_mut(&entry.ups_name) {
          info!(
            message = "device details updated",
            namespace = %self.state.namespace,
            device = %device.name
          );

          events.updated_device(entry.ups_name);
          device.desc = entry.desc;
        }
      }

      for device_name in diff.deleted.into_iter() {
        info!(
          message = "device disconnected",
          namespace = %self.state.namespace,
          device = %device_name
        );

        _ = write_lock.devices.remove(&device_name);
        events.removed_device(device_name);
      }

      if write_lock.status != ConnectionStatus::Online {
        info!(
          message = "ups daemon is online",
          namespace = %self.state.namespace
        );

        write_lock.status = ConnectionStatus::Online;
        events.set_upsd_status(ConnectionStatus::Online);
      }

      write_lock.last_device_sync = Some(Utc::now());
      write_lock.prot_ver = Some(prot_ver.value.into_boxed_str());
      write_lock.ver = Some(ver.value.into_boxed_str());

      if let Err(err) = events.send(&self.event_channel) {
        warn!(
          message = "unable to send events",
          namespace = %self.state.namespace,
          reason = %err
        );
      }

      Ok(())
    }
  }

  async fn load_device_entry(
    client: NutPoolClient,
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
          warn!(
            message = "failed to get RW variable type details, variable will be displayed as read-only",
            device = %err.name,
            reason = %err.inner
          );
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
    client: NutPoolClient,
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
            warn!(
              message = "nut driver reports variable type as enum, but it does not provide any enum option",
              var_name = %var_name,
              device = %ups_name
            );
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
              warn!(
                message = "nut driver reports variable type as range, but it does not provide any range information",
                var_name = %var_name,
                device = %ups_name
              );

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
