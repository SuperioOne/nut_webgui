use super::UpsVarDetail;
use crate::{
  ups_service::{UpsDetails, UpsUpdateMessage},
  upsd_client::{
    client::{Client, UpsClient},
    errors::NutClientErrors,
    ups_variables::{UpsVariable, VAR_UPS_STATUS},
  },
};
use std::{io::ErrorKind, time::Duration};
use tokio::{
  net::ToSocketAddrs,
  select, spawn,
  sync::mpsc::Sender,
  task::JoinHandle,
  time::{interval, Instant, Interval, MissedTickBehavior},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument, warn};

#[derive(Debug)]
pub struct UpsPollerConfig {
  pub address: String,
  pub write_channel: Sender<UpsUpdateMessage>,
  pub poll_freq: Duration,
  pub poll_interval: Duration,
  pub cancellation: CancellationToken,
}

#[derive(Debug)]
enum PollServiceError {
  ClientError(NutClientErrors),
  ChannelError,
}

impl From<NutClientErrors> for PollServiceError {
  fn from(value: NutClientErrors) -> Self {
    PollServiceError::ClientError(value)
  }
}

struct UpsPollInterval {
  interval: Interval,
  last_full_sync: Option<Instant>,
  full_sync_period: Duration,
}

enum UpsPollType {
  Full,
  Partial,
}

impl UpsPollInterval {
  pub fn new(poll_interval: Duration, poll_freq: Duration) -> Self {
    let mut interval = interval(poll_interval);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    Self {
      interval,
      last_full_sync: None,
      full_sync_period: poll_freq,
    }
  }

  #[inline]
  pub async fn tick(&mut self) -> UpsPollType {
    let instant = self.interval.tick().await;

    match self.last_full_sync {
      Some(last_sync) => {
        if instant.duration_since(last_sync) >= self.full_sync_period {
          self.last_full_sync = Some(instant);
          UpsPollType::Full
        } else {
          UpsPollType::Partial
        }
      }
      None => {
        self.last_full_sync = Some(instant);
        UpsPollType::Full
      }
    }
  }

  pub fn schedule_full_sync(&mut self) {
    // Resets scheduler internal tracker for full sync
    self.last_full_sync = None;
  }
}

#[instrument(name = "ups_poll_service")]
pub fn ups_poll_service(config: UpsPollerConfig) -> JoinHandle<()> {
  spawn(async move {
    let UpsPollerConfig {
      address,
      poll_freq,
      poll_interval,
      write_channel,
      cancellation,
    } = config;

    let mut should_reconnect = false;
    let mut client = UpsClient::create(&address)
      .await
      .expect("Cannot connect to the UPS daemon service.");

    info!(
      message = "Connected to UPS daemon service.",
      address = &address
    );

    let mut poll_scheduler = UpsPollInterval::new(poll_interval, poll_freq);

    'MAIN_LOOP: while !cancellation.is_cancelled() {
      // wait for next tick or cancellation (SIGINT)
      let poll_type = select! {
        _ = cancellation.cancelled() => {
          break 'MAIN_LOOP;
        }
        poll_type = poll_scheduler.tick() => {
          poll_type
        }
      };

      let update_result = match poll_type {
        UpsPollType::Full => poll_ups_full(&mut client, &write_channel).await,
        UpsPollType::Partial => poll_ups_partial(&mut client, &write_channel).await,
      };

      match update_result {
        Err(PollServiceError::ClientError(NutClientErrors::IOError(err))) => match err {
          ErrorKind::WouldBlock => {}
          ErrorKind::PermissionDenied => {
            error!("TCP connection permission denied.");
            panic!("Client cannot create TCP stream.");
          }
          ErrorKind::AddrNotAvailable => {
            error!(
              message = "Configured address does not exists or unreachable.",
              address = address
            );

            panic!("Client cannot create TCP stream.");
          }
          error_kind => {
            warn!(
              message = "Client connection failed with an error.",
              error_kind = error_kind.to_string()
            );

            should_reconnect = true;
          }
        },
        Err(PollServiceError::ChannelError) => {
          info!(
            "Sending updates to memory store channel failed, receiver channel is disconnected."
          );
        }
        Err(err) => {
          error!(
            message = "Poll service error occurred",
            reason = format!("{:?}", err)
          );

          // Clear all UPS info stored in memory
          if let Err(err) = write_channel
            .send(UpsUpdateMessage::FullUpdate {
              content: Vec::new(),
            })
            .await
          {
            error!(
              message =
                "Sending updates to memory store channel failed, receiver channel is disconnected.",
              reason = err.to_string()
            );
          }

          poll_scheduler.schedule_full_sync();
        }
        _ => {}
      };

      if should_reconnect && !cancellation.is_cancelled() {
        should_reconnect = false;
        poll_scheduler.schedule_full_sync();

        if let Err(err) = client.reconnect().await {
          // Log connection issue without panicking since it might be temporary connection issue.
          error!("UPS daemon re-connection failed: {:?}", err);
        } else {
          info!("Reconnected to the UPS daemon service.");
        }
      }
    }

    if let Err(err) = client.close().await {
      error!("NUT Client shutdown failed. {:?}", err);
    }

    drop(write_channel);
    info!("Poll service shutdown.");
  })
}

#[inline]
async fn poll_ups_full<A>(
  client: &mut UpsClient<A>,
  channel: &Sender<UpsUpdateMessage>,
) -> Result<(), PollServiceError>
where
  A: ToSocketAddrs,
{
  debug!("Refreshing the all available Ups devices.");
  let ups_list = client.get_ups_list().await?;
  let mut content: Vec<UpsDetails> = Vec::with_capacity(ups_list.len());

  for ups in ups_list {
    let ups_details = get_ups_details(client, &ups.name).await;

    match ups_details {
      Ok((commands, variables)) => {
        let details = UpsDetails {
          commands,
          desc: ups.desc.clone(),
          name: ups.name.clone(),
          variables,
        };

        content.push(details);

        debug!(message = "UPS info received.", ups_name = &ups.name);
      }
      Err(err) => {
        error!(
          message = "Unable to get UPS details",
          ups_name = &ups.name,
          reason = format!("{:?}", err)
        );
      }
    };
  }

  let update_message = UpsUpdateMessage::FullUpdate { content };

  debug!(message = "UPS full update queued.");

  channel
    .send(update_message)
    .await
    .map_err(|_| PollServiceError::ChannelError)?;

  Ok(())
}

#[inline]
async fn poll_ups_partial<A>(
  client: &mut UpsClient<A>,
  channel: &Sender<UpsUpdateMessage>,
) -> Result<(), PollServiceError>
where
  A: ToSocketAddrs,
{
  let ups_list = client.get_ups_list().await?;
  let mut content: Vec<UpsVarDetail> = Vec::with_capacity(ups_list.len());

  for ups in ups_list {
    match client.get_var(&ups.name, VAR_UPS_STATUS).await {
      Ok(ups_status) => {
        let detail = UpsVarDetail {
          name: ups.name.clone(),
          variable: ups_status,
        };

        content.push(detail);

        debug!(message = "Ups status update queued.", ups_name = &ups.name);
      }
      Err(err) => {
        error!(
          message = "Unable to get UPS status info",
          ups_name = &ups.name,
          reason = format!("{:?}", err)
        );
      }
    };
  }

  let update_message = UpsUpdateMessage::PartialUpdate { content };

  channel
    .send(update_message)
    .await
    .map_err(|_| PollServiceError::ChannelError)?;

  Ok(())
}

#[inline]
async fn get_ups_details<A>(
  client: &mut UpsClient<A>,
  ups_name: &str,
) -> Result<(Vec<Box<str>>, Vec<UpsVariable>), NutClientErrors>
where
  A: ToSocketAddrs,
{
  let commands = client.get_cmd_list(ups_name).await?;
  let variables = client.get_var_list(ups_name).await?;

  Ok((commands, variables))
}
