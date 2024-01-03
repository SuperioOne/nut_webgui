use std::io::ErrorKind;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::{select, spawn};
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio::time::{sleep};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument, warn};
use crate::ups_service::UpsUpdateMessage;
use crate::upsd_client::client::{Client, UpsClient};
use crate::upsd_client::{Cmd, Var};
use crate::upsd_client::errors::{NutClientErrors};

#[derive(Debug)]
pub struct UpsPollerConfig {
  pub address: SocketAddr,
  pub poll_freq: Duration,
  pub write_channel: Sender<UpsUpdateMessage>,
  pub cancellation: CancellationToken,
}

#[derive(Debug)]
enum PollServiceError {
  ClientError(NutClientErrors),
  ChannelError,
}

#[instrument(name = "ups_poll_service")]
pub fn ups_poller_service(config: UpsPollerConfig) -> JoinHandle<()> {
  spawn(async move {
    let UpsPollerConfig {
      address,
      poll_freq,
      write_channel,
      cancellation
    } = config;

    let mut should_reconnect = false;
    let mut client = UpsClient::create(address).await.expect("Cannot connect to the UPS daemon service.");
    info!(message = "Connected to UPS daemon service.", address = address.to_string());

    while !cancellation.is_cancelled() {
      match update_ups_store(&mut client, &write_channel).await {
        Err(PollServiceError::ClientError(NutClientErrors::IOError(err))) => {
          match err {
            ErrorKind::WouldBlock => {}
            ErrorKind::PermissionDenied => {
              error!("TCP connection permission denied.");
              panic!("Client cannot create TCP stream");
            }
            ErrorKind::AddrNotAvailable => {
              error!("Configured address {0} does not exists or unreachable.", address);
              panic!("Client cannot create TCP stream");
            }
            error_kind => {
              warn!("Client connection failed with error kind: {}", error_kind);
              should_reconnect = true;
            }
          }
        }
        Err(PollServiceError::ChannelError) => {
          info!("Sending updates to memory store channel failed, receiver channel is disconnected.");
        }
        Err(err) => {
          error!("Poll service error occurred: {:?}", err);
        }
        _ => {}
      };

      if should_reconnect && !cancellation.is_cancelled() {
        should_reconnect = false;
        if let Err(err) = client.reconnect().await {
          // Log connection issue without panicking since it might be temporary connection issue.
          error!("UPS daemon re-connection failed: {:?}", err);
        } else {
          info!("Reconnected to the UPS daemon service.");
        }
      }

      // wait for next tick or cancellation (SIGINT)
      select! {
        _ = cancellation.cancelled() => {
          break;
        }
        _ = sleep(poll_freq) => {
          continue;
        }
      }
    }

    if let Err(err) = client.close().await {
      error!("NUT Client shutdown failed. {:?}", err);
    }
    drop(write_channel);
    info!("Poll service service shutdown.");
  })
}

async fn update_ups_store(client: &mut UpsClient, channel: &Sender<UpsUpdateMessage>) -> Result<(), PollServiceError> {
  let ups_list = client.get_ups_list().await
    .map_err(|err| { PollServiceError::ClientError(err) })?;

  debug!("UPS list fetched.");
  for ups in ups_list {
    let cmd = client.get_cmd_list(ups.name.as_ref()).await
      .map_err(|err| PollServiceError::ClientError(err))?;
    let vars = client.get_var_list(ups.name.as_ref()).await
      .map_err(|err| PollServiceError::ClientError(err))?;

    debug!(message = "UPS details parsed.", ups_name = ups.name.as_ref());

    let update_message = UpsUpdateMessage {
      commands: cmd.into_iter().map(|Cmd { name: _, cmd }| cmd).collect(),
      name: ups.name,
      desc: ups.desc,
      variables: vars.into_iter().map(|Var { name: _, var }| var).collect(),
    };

    channel.send(update_message).await
      .map_err(|_| { PollServiceError::ChannelError })?;
  }

  Ok(())
}
