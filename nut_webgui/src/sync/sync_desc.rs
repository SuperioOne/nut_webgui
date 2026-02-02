use crate::{
  background_service::BackgroundService,
  event::{SystemEvent, channel::EventChannel},
  state::ServerState,
};
use futures::future::join_all;
use nut_webgui_upsmc::{
  CmdName, UpsName, VarName,
  client::{AsyncNutClient, NutPoolClient},
  response::{CmdDesc, UpsVarDesc},
};
use std::{collections::HashSet, sync::Arc};
use tokio::{join, select, sync::broadcast::error::RecvError, task::JoinSet};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

pub struct DescriptionSyncService {
  event_channel: EventChannel,
  state: Arc<ServerState>,
}

struct DescriptionTask {
  state: Arc<ServerState>,
}

struct TaskContext {
  name: UpsName,
  cmds: Vec<CmdName>,
  vars: Vec<VarName>,
}

impl DescriptionSyncService {
  pub fn new(event_channel: EventChannel, state: Arc<ServerState>) -> Self {
    Self {
      state,
      event_channel,
    }
  }
}

impl BackgroundService for DescriptionSyncService {
  fn run(
    &self,
    token: CancellationToken,
  ) -> core::pin::Pin<Box<dyn core::future::Future<Output = ()> + Send>> {
    let mut events = self.event_channel.subscribe();
    let state = self.state.clone();

    Box::pin(async move {
      let task = DescriptionTask { state };

      'MAIN: loop {
        select! {
            event = events.recv() => {
              match event.as_deref() {
                Ok(SystemEvent::DeviceAddition { devices, namespace }) => {
                  task.next(devices, namespace).await;
                },
                Ok(_) => continue,
                Err(RecvError::Closed) => break 'MAIN,
                Err(RecvError::Lagged(lagged)) => {
                  warn!(
                    message = "description service can't keep up with system events",
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

      debug!(message = "description sync task stopped");
    })
  }
}

impl DescriptionTask {
  pub async fn next(&self, devices: &[UpsName], namespace: &str) {
    let upsd_state = match self.state.upsd_servers.get(namespace) {
      Some(upsd) => upsd,
      None => {
        warn!(
          message = "cannot sync descriptions, namespace does not exists",
          namespace = %namespace
        );

        return;
      }
    };

    let task_ctx: Vec<TaskContext> = {
      let mut tmp_lookup = HashSet::new();
      let mut ctxs = Vec::new();
      let upsd_lock = upsd_state.daemon_state.read().await;
      let shared_desc_lock = self.state.shared_desc.read().await;

      for name in devices {
        match upsd_lock.devices.get(name) {
          Some(entry) => {
            let mut cmds: Vec<CmdName> = Vec::new();
            let mut vars: Vec<VarName> = Vec::new();

            for (var_name, _) in entry.variables.iter() {
              let name = var_name.as_str();

              if !shared_desc_lock.contains_key(name) && !tmp_lookup.contains(name) {
                _ = tmp_lookup.insert(var_name.as_str());
                vars.push(var_name.clone());
              }
            }

            for cmd in entry.commands.iter() {
              let name = cmd.as_str();

              if !shared_desc_lock.contains_key(name) && !tmp_lookup.contains(name) {
                _ = tmp_lookup.insert(cmd.as_str());
                cmds.push(cmd.clone());
              }
            }

            if !cmds.is_empty() || !vars.is_empty() {
              ctxs.push(TaskContext {
                name: name.clone(),
                cmds,
                vars,
              })
            }
          }
          None => {
            debug!(
              message = "ignoring description sync, device is already removed from server state",
              namespace = %namespace,
              device_name = %name
            );
          }
        }
      }

      ctxs
    };

    if !task_ctx.is_empty() {
      let mut task_set = JoinSet::new();

      for ctx in task_ctx {
        let nut_client = upsd_state.connection_pool.clone();
        task_set.spawn(Self::load_descs(nut_client, ctx));
      }

      let results = task_set.join_all().await;
      let mut shared_desc_lock = self.state.shared_desc.write().await;

      for (k, v) in results.into_iter().flatten() {
        _ = shared_desc_lock.insert(k.into(), v)
      }
    }
  }

  /// **concurrently** loads requested command and variable descriptions for target ups.
  async fn load_descs(client: NutPoolClient, ctx: TaskContext) -> Vec<(Box<str>, Box<str>)> {
    let cmd_future = join_all(ctx.cmds.iter().map(|v| client.get_cmd_desc(&ctx.name, v)));
    let var_future = join_all(ctx.vars.iter().map(|v| client.get_var_desc(&ctx.name, v)));
    let (cmds, vars) = join!(cmd_future, var_future);
    let mut results = Vec::with_capacity(cmds.len() + vars.len());

    for CmdDesc { cmd, desc, .. } in cmds.into_iter().flatten() {
      results.push((cmd.into_boxed_str(), desc));
    }

    for UpsVarDesc { name, desc, .. } in vars.into_iter().flatten() {
      results.push((name.into_box_str(), desc));
    }

    results
  }
}
