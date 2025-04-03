use super::UpsVarDetail;
use nut_webgui_upsmc::{
  Client, UpsClient,
  errors::NutClientErrors,
  ups_variables::{UpsVariable, VAR_UPS_STATUS},
};

use crate::ups_services::{UpsDetails, UpsUpdateMessage};
use std::{fmt::Display, io::ErrorKind, time::Duration};
use tokio::{
  net::ToSocketAddrs,
  select, spawn,
  sync::mpsc::Sender,
  task::JoinHandle,
  time::{Instant, Interval, MissedTickBehavior, interval},
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

impl Display for PollServiceError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let message = match self {
      PollServiceError::ClientError(nut_client_errors) => &nut_client_errors.to_string(),
      PollServiceError::ChannelError => "Internal mpsc channel failed.",
    };

    f.write_str(message)
  }
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

#[derive(Debug, Clone, Copy)]
enum UpsPollType {
  Full,
  Partial,
}

impl Display for UpsPollType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      UpsPollType::Full => f.write_str("Full"),
      UpsPollType::Partial => f.write_str("Partial"),
    }
  }
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

  /// Resets internal tracker for full sync
  pub fn schedule_full_sync(&mut self) {
    self.last_full_sync = None;
  }
}

#[instrument(name = "upsd_poll_service")]
pub fn upsd_poll_service(config: UpsPollerConfig) -> JoinHandle<()> {
  spawn(async move {
    let UpsPollerConfig {
      address,
      poll_freq,
      poll_interval,
      write_channel,
      cancellation,
    } = config;

    // Thread local, lock free working device name list to orchestrate `poll_ups_partial`.
    let mut devices: Vec<Box<str>> = Vec::new();
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
        UpsPollType::Full => poll_ups_full(&mut devices, &mut client, &write_channel).await,
        UpsPollType::Partial => poll_ups_partial(&mut devices, &mut client, &write_channel).await,
      };

      match update_result {
        Err(PollServiceError::ClientError(NutClientErrors::IOError { kind })) => match kind {
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
              error_kind = %error_kind
            );

            mark_as_dead(&write_channel).await;
            should_reconnect = true;
            poll_scheduler.schedule_full_sync();
          }
        },
        Err(PollServiceError::ChannelError) => {
          info!("Sending updates to channel failed, receiver channel is disconnected.");
        }
        Err(err) => {
          error!(
            message = "Poll service error occurred.",
            reason = %err
          );

          mark_as_dead(&write_channel).await;
          poll_scheduler.schedule_full_sync();
        }
        Ok(()) => {
          debug!(
            message = "Poll interval completed.",
            poll_type = %poll_type
          );
        }
      };

      if should_reconnect && !cancellation.is_cancelled() {
        should_reconnect = false;
        poll_scheduler.schedule_full_sync();

        if let Err(err) = client.reconnect().await {
          // Log connection issue without panicking since it might be temporary connection issue.
          error!(message = "UPS daemon re-connection attempt failed.", reason = %err);
        } else {
          info!("Reconnected to the UPS daemon service.");
        }
      }
    }

    if let Err(err) = client.close().await {
      error!(message = "NUT Client shutdown failed.", reason = %err);
    }

    drop(write_channel);
    info!("Upsd poll service shutdown.");
  })
}

#[inline]
async fn poll_ups_full<A>(
  devices: &mut Vec<Box<str>>,
  client: &mut UpsClient<A>,
  channel: &Sender<UpsUpdateMessage>,
) -> Result<(), PollServiceError>
where
  A: ToSocketAddrs,
{
  debug!("Refreshing the all available Ups devices.");
  let ups_list = client.get_ups_list().await?;
  let mut data: Vec<UpsDetails> = Vec::with_capacity(ups_list.len());
  let mut override_devices = false;

  for ups in ups_list.into_iter() {
    let ups_details: Result<(Vec<Box<str>>, Vec<UpsVariable>), NutClientErrors> = {
      async {
        let commands = client.get_cmd_list(&ups.name).await?;
        let variables = client.get_var_list(&ups.name).await?;

        Ok((commands, variables))
      }
    }
    .await;

    match ups_details {
      Ok((commands, variables)) => {
        let details = UpsDetails {
          commands,
          desc: ups.desc,
          name: ups.name,
          variables,
        };

        debug!(message = "UPS info received.", ups_name = &details.name);

        data.push(details);
      }
      Err(err) => {
        override_devices = true;

        error!(
          message = "UPS is listed by daemon, but unable to get UPS details.",
          ups_name = &ups.name,
          reason = %err
        );
      }
    };
  }

  if override_devices || devices.is_empty() {
    *devices = data.iter().map(|ups| ups.name.clone()).collect();
  }

  channel
    .send(UpsUpdateMessage::FullUpdate { data })
    .await
    .map_err(|_| PollServiceError::ChannelError)?;

  debug!(message = "UPS full update queued.");

  Ok(())
}

#[inline]
async fn poll_ups_partial<A>(
  devices: &mut Vec<Box<str>>,
  client: &mut UpsClient<A>,
  channel: &Sender<UpsUpdateMessage>,
) -> Result<(), PollServiceError>
where
  A: ToSocketAddrs,
{
  let mut data: Vec<UpsVarDetail> = Vec::new();
  let mut override_devices = false;

  for ups in devices.iter() {
    match client.get_var(ups, VAR_UPS_STATUS).await {
      Ok(ups_status) => {
        let detail = UpsVarDetail {
          name: ups.clone(),
          variable: ups_status,
        };

        data.push(detail);

        debug!(message = "Ups status update queued.", ups_name = &ups);
      }
      Err(err) => {
        override_devices = true;

        error!(
          message =
            "Unable to get UPS status info. Deferring status update until the next full sync.",
          ups_name = &ups,
          reason = %err
        );
      }
    };
  }

  if override_devices {
    *devices = data.iter().map(|ups| ups.name.clone()).collect();
  }

  channel
    .send(UpsUpdateMessage::PartialUpdate { data })
    .await
    .map_err(|_| PollServiceError::ChannelError)?;

  Ok(())
}

#[inline]
async fn mark_as_dead(channel: &Sender<UpsUpdateMessage>) {
  if let Err(_) = channel.send(UpsUpdateMessage::MarkAsDead).await {
    warn!("Unable to mark daemon state as dead. Message channel is closed.");
  }
}
