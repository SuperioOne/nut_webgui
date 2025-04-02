use crate::{
  ups_daemon_state::{DaemonStatus, UpsDaemonState, UpsEntry},
  ups_services::UpsUpdateMessage,
};
use chrono::Utc;
use nut_webgui_upsmc::ups_variables::UpsVariable;
use std::{collections::HashMap, sync::Arc};
use tokio::{
  spawn,
  sync::{RwLock, mpsc::Receiver},
  task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tracing::{info, instrument, warn};

#[derive(Debug)]
pub struct UpsStorageConfig {
  pub read_channel: Receiver<UpsUpdateMessage>,
  pub cancellation: CancellationToken,
  pub upsd_state: Arc<RwLock<UpsDaemonState>>,
}

#[instrument(name = "upsd_state_service")]
pub fn upsd_state_service(config: UpsStorageConfig) -> JoinHandle<()> {
  spawn(async move {
    let UpsStorageConfig {
      upsd_state,
      mut read_channel,
      cancellation,
    } = config;

    while !cancellation.is_cancelled() {
      match read_channel.recv().await {
        Some(UpsUpdateMessage::PartialUpdate { data }) => {
          let mut state = upsd_state.write().await;

          for item in data.into_iter() {
            match state.get_ups_mut(&item.name) {
              Some(ups_entry) => {
                let old_var = {
                  let mut e: Option<&mut UpsVariable> = None;

                  for var in ups_entry.variables.iter_mut() {
                    // TODO: Check UpsVariable enum marker byte instead of whole name
                    if var.name() == item.variable.name() {
                      e = Some(var);
                      break;
                    }
                  }

                  e
                };

                if let Some(old_var) = old_var {
                  *old_var = item.variable;
                } else {
                  ups_entry.variables.push(item.variable);
                }

                state.last_modified = Some(Utc::now());
              }
              None => warn!(
                "Partial update ignored. Ups {} does not exists anymore.",
                item.name
              ),
            }
          }
        }
        Some(UpsUpdateMessage::FullUpdate { data }) => {
          let mut new_map: HashMap<Box<str>, UpsEntry> = HashMap::new();

          for item in data.into_iter() {
            let key = item.name.clone();
            let entry = UpsEntry {
              commands: item.commands,
              variables: item.variables,
              name: item.name,
              desc: item.desc,
            };

            new_map.insert(key, entry);
          }

          let mut state = upsd_state.write().await;
          let now = Utc::now();

          state.ups_list = new_map;
          state.last_full_sync = Some(now);
          state.last_modified = Some(now);
          state.status = DaemonStatus::Online;
        }
        Some(UpsUpdateMessage::MarkAsDead) => {
          let mut state = upsd_state.write().await;
          state.reset_with_status(DaemonStatus::Dead);
        }

        None => {}
      };
    }

    info!("Upsd state service is shutdown.");
  })
}
