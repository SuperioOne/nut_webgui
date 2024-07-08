use crate::{
  ups_mem_store::{UpsEntry, UpsStore},
  ups_service::UpsUpdateMessage,
  upsd_client::ups_variables::UpsVariable,
};
use std::sync::Arc;
use tokio::{
  spawn,
  sync::{mpsc::Receiver, RwLock},
  task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tracing::{info, instrument, warn};

#[derive(Debug)]
pub struct UpsStorageConfig {
  pub read_channel: Receiver<UpsUpdateMessage>,
  pub cancellation: CancellationToken,
  pub store: Arc<RwLock<UpsStore>>,
}

#[instrument(name = "ups_storage_service")]
pub fn ups_storage_service(config: UpsStorageConfig) -> JoinHandle<()> {
  spawn(async move {
    let UpsStorageConfig {
      store,
      mut read_channel,
      cancellation,
    } = config;

    while !cancellation.is_cancelled() {
      match read_channel.recv().await {
        Some(UpsUpdateMessage::PartialUpdate { name, variable }) => {
          match store.write().await.get_mut(&name) {
            Some(ups_entry) => {
              let old_var = {
                let mut e: Option<&mut UpsVariable> = None;

                for var in ups_entry.variables.iter_mut() {
                  if *var.name() == *name {
                    e = Some(var);
                    break;
                  }
                }

                e
              };

              if let Some(old_var) = old_var {
                *old_var = variable;
              } else {
                ups_entry.variables.push(variable);
              }
            }
            None => warn!(
              "Partial update ignored. Ups {} does not exists anymore.",
              name
            ),
          }
        }
        Some(UpsUpdateMessage::FullUpdate {
          name,
          desc,
          commands,
          variables,
        }) => {
          let entry = UpsEntry {
            desc,
            name,
            variables,
            commands,
          };

          store.write().await.put(entry);
        }
        None => {}
      };
    }

    info!("Storage service shutdown.");
  })
}
