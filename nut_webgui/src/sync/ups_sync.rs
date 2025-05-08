use crate::state::{DeviceEntry, ServerState};
use nut_webgui_upsmc::{
  UpsName, VarName,
  clients::{AsyncNutClient, NutPoolClient},
  responses::{UpsDevice, UpsList},
  ups_status::UpsStatus,
};
use std::{collections::HashSet, str::FromStr, sync::Arc, time::Duration};
use tokio::{join, net::ToSocketAddrs, select, sync::RwLock, task::JoinSet, time::interval};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

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
    state: Arc<RwLock<ServerState>>,
    poll_interval: Duration,
    cancellation: CancellationToken,
  ) -> Self {
    Self {
      task: SyncTask { client, state },
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

    'MAIN: loop {
      select! {
        _ = interval.tick() => { }
        _ = cancellation.cancelled() =>  {
            break 'MAIN;
        }
      };

      select! {
        v = task.next() => { match v {
            Ok(_) => {},
            Err(err) => error!(message = "inner call failed", err=%err),
        } }
        _ = cancellation.cancelled() =>  {
            break 'MAIN;
        }
      };
    }

    debug!(message = "ups sync schedule completed");
  }
}

struct SyncTask<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  client: NutPoolClient<A>,
  state: Arc<RwLock<ServerState>>,
}

impl<A> SyncTask<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  pub async fn next(&self) -> Result<(), nut_webgui_upsmc::errors::Error> {
    let response = self.client.list_ups().await?;

    let existing_devices: HashSet<_> = {
      let state_lock = self.state.read().await;
      state_lock.devices.keys().cloned().collect()
    };

    let DiffResult { new, update } = get_diff(existing_devices, response);

    let mut entry_set = JoinSet::new();

    for device in new.into_iter() {
      let client = self.client.clone();
      entry_set.spawn(load_device_entry(client, device));
    }

    let mut entries: Vec<DeviceEntry> = Vec::new();

    while let Some(result) = entry_set.join_next_with_id().await {
      match result {
        Ok((_, Ok(entry))) => entries.push(entry),
        Ok((id, Err(err))) => {
          error!(message = "unable to load device details", err = %err, id = %id)
        }
        Err(err) => error!(message = "unable to join load device entry task", err = %err),
      }
    }

    {
      let mut write_lock = self.state.write().await;
      for entry in entries.into_iter() {
        write_lock.devices.insert(entry.name.clone(), entry);
      }
    }

    Ok(())
  }
}

struct DiffResult {
  update: Vec<UpsDevice>,
  new: Vec<UpsDevice>,
}

fn get_diff(existing: HashSet<UpsName>, mut response: UpsList) -> DiffResult {
  let mut result = DiffResult {
    new: Vec::new(),
    update: Vec::new(),
  };

  while let Some(device) = response.devices.pop() {
    if existing.contains(&device.ups_name) {
      result.update.push(device);
    } else {
      result.new.push(device)
    }
  }

  result
}

async fn load_device_entry<A>(
  client: NutPoolClient<A>,
  device: UpsDevice,
) -> Result<DeviceEntry, nut_webgui_upsmc::errors::Error>
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

  let variables = vars?.variables;
  let status = match variables.get(&VarName::UPS_STATUS) {
    Some(nut_webgui_upsmc::Value::String(inner)) => UpsStatus::from_str(inner).unwrap_or_default(),
    _ => UpsStatus::default(),
  };

  let entry = DeviceEntry {
    name: ups_name,
    desc,
    attached: clients?.ips,
    commands: commands?.cmds,
    variables,
    rw_variables: rw_vars?.variables.iter().map(|(k, _)| k.clone()).collect(), // FIX: RW need it's own type. It can be enum, range etc
    status,
  };

  debug!(message = "new ups entry", entry = ?&entry.name);

  Ok(entry)
}
