use crate::ups_mem_store::{UpsEntry, UpsStore};
use crate::ups_service::UpsUpdateMessage;
use std::sync::Arc;
use tokio::spawn;
use tokio::sync::mpsc::Receiver;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{info, instrument};

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
      if let Some(message) = read_channel.recv().await {
        let UpsUpdateMessage {
          name,
          commands,
          variables,
          desc,
        } = message;
        let entry = UpsEntry {
          desc,
          name,
          variables,
          commands,
        };

        store.write().await.create_or_update(entry);
      }
    }

    info!("Storage service shutdown.");
  })
}
