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
        Some(UpsUpdateMessage::PartialUpdate { content }) => {
          let mut store_handle = store.write().await;

          for item in content.into_iter() {
            match store_handle.get_mut(&item.name) {
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
              }
              None => warn!(
                "Partial update ignored. Ups {} does not exists anymore.",
                item.name
              ),
            }
          }
        }
        Some(UpsUpdateMessage::FullUpdate { content }) => {
          let mut new_store = UpsStore::new();

          for item in content.into_iter() {
            let entry = UpsEntry {
              commands: item.commands,
              variables: item.variables,
              name: item.name,
              desc: item.desc,
            };

            new_store.put(entry);
          }

          let mut store_ptr = store.write().await;

          // Swap old memory with the fresh one
          *store_ptr = new_store;
        }
        None => {}
      };
    }

    info!("Storage service shutdown.");
  })
}
