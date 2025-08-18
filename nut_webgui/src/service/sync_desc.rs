use super::BackgroundService;
use crate::{
  event::{EventChannel, SystemEvent},
  state::ServerState,
};
use nut_webgui_upsmc::{
  CmdName, UpsName, VarName,
  clients::{AsyncNutClient, NutPoolClient},
  responses::{CmdDesc, UpsVarDesc},
};
use std::{collections::HashSet, net::ToSocketAddrs, sync::Arc};
use tokio::{
  join, select,
  sync::{RwLock, broadcast::error::RecvError},
  task::JoinSet,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

pub struct DescriptionSyncService<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  event_channel: EventChannel,
  client: NutPoolClient<A>,
  state: Arc<RwLock<ServerState>>,
}

impl<A> DescriptionSyncService<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  pub fn new(
    client: NutPoolClient<A>,
    event_channel: EventChannel,
    state: Arc<RwLock<ServerState>>,
  ) -> Self {
    Self {
      client,
      state,
      event_channel,
    }
  }
}

impl<A> BackgroundService for DescriptionSyncService<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  fn run(
    &self,
    token: CancellationToken,
  ) -> core::pin::Pin<Box<dyn core::future::Future<Output = ()> + Send + Sync + 'static>> {
    let mut events = self.event_channel.subscribe();
    let client = self.client.clone();
    let state = self.state.clone();

    Box::pin(async move {
      let task = DescriptionTask { state, client };

      'MAIN: loop {
        select! {
            event = events.recv() => {
              match event {
                Ok(SystemEvent::DeviceAddition { devices }) => {
                  task.next(devices).await;
                },
                Ok(_) => continue,
                Err(RecvError::Closed) => break 'MAIN,
                Err(RecvError::Lagged(lagged)) => {
                  warn!(message = "description service can't keep up with system events", lagged_event_count=lagged)
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

struct DescriptionTask<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  client: NutPoolClient<A>,
  state: Arc<RwLock<ServerState>>,
}

struct TaskContext {
  name: UpsName,
  cmds: Vec<CmdName>,
  vars: Vec<VarName>,
}

impl<A> DescriptionTask<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  pub async fn next(&self, devices: Vec<UpsName>) {
    let task_ctx: Vec<TaskContext> = {
      let mut tmp_lookup = HashSet::new();
      let mut ctxs = Vec::with_capacity(devices.len());
      let read_lock = self.state.read().await;

      for name in devices {
        match read_lock.devices.get(&name) {
          Some(entry) => {
            let mut cmds: Vec<CmdName> = Vec::new();
            let mut vars: Vec<VarName> = Vec::new();

            for (var_name, _) in entry.variables.iter() {
              let name = var_name.as_str();

              if !read_lock.shared_desc.contains_key(name) && !tmp_lookup.contains(name) {
                _ = tmp_lookup.insert(var_name.as_str());
                vars.push(var_name.clone());
              }
            }

            for cmd in entry.commands.iter() {
              let name = cmd.as_str();

              if !read_lock.shared_desc.contains_key(name) && !tmp_lookup.contains(name) {
                _ = tmp_lookup.insert(cmd.as_str());
                cmds.push(cmd.clone());
              }
            }

            if !cmds.is_empty() || !vars.is_empty() {
              ctxs.push(TaskContext { name, cmds, vars })
            }
          }
          None => {
            debug!(
              message = "ignoring description sync, device is already removed from server state",
              device_name = %name
            );
          }
        }
      }

      ctxs
    };

    let mut task_set = JoinSet::new();

    for ctx in task_ctx {
      let nut_client = self.client.clone();
      task_set.spawn(Self::load_descs(nut_client, ctx));
    }

    let results = task_set.join_all().await;
    let mut write_lock = self.state.write().await;

    for (k, v) in results.into_iter().flatten() {
      _ = write_lock.shared_desc.insert(k.into(), v)
    }
  }

  /// **concurrently** loads requested command and variable descriptions for target ups.
  async fn load_descs(client: NutPoolClient<A>, ctx: TaskContext) -> Vec<(Box<str>, Box<str>)> {
    let cmd_future =
      futures::future::join_all(ctx.cmds.iter().map(|v| client.get_cmd_desc(&ctx.name, v)));

    let var_future =
      futures::future::join_all(ctx.vars.iter().map(|v| client.get_var_desc(&ctx.name, v)));

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
