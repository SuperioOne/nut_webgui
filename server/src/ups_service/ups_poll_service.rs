use crate::{
  ups_service::UpsUpdateMessage,
  upsd_client::{
    client::{Client, UpsClient},
    errors::NutClientErrors,
    ups_variables::VAR_UPS_STATUS,
    Cmd, Ups, Var,
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
}

#[instrument(name = "ups_poll_service")]
pub fn ups_polling_service(config: UpsPollerConfig) -> JoinHandle<()> {
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
    let mut ups_list: Vec<Ups> = Vec::new();

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
        UpsPollType::Full => poll_ups_full(&mut ups_list, &mut client, &write_channel).await,
        UpsPollType::Partial => poll_ups_partial(&ups_list, &mut client, &write_channel).await,
      };

      match update_result {
        Err(PollServiceError::ClientError(NutClientErrors::IOError(err))) => match err {
          ErrorKind::WouldBlock => {}
          ErrorKind::PermissionDenied => {
            error!("TCP connection permission denied.");
            panic!("Client cannot create TCP stream");
          }
          ErrorKind::AddrNotAvailable => {
            error!(
              "Configured address {0} does not exists or unreachable.",
              address
            );
            panic!("Client cannot create TCP stream");
          }
          error_kind => {
            warn!("Client connection failed with error kind: {}", error_kind);
            should_reconnect = true;
          }
        },
        Err(PollServiceError::ChannelError) => {
          info!(
            "Sending updates to memory store channel failed, receiver channel is disconnected."
          );
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
    }

    if let Err(err) = client.close().await {
      error!("NUT Client shutdown failed. {:?}", err);
    }
    drop(write_channel);
    info!("Poll service service shutdown.");
  })
}

#[inline]
async fn poll_ups_full<A>(
  ups_list: &mut Vec<Ups>,
  client: &mut UpsClient<A>,
  channel: &Sender<UpsUpdateMessage>,
) -> Result<(), PollServiceError>
where
  A: ToSocketAddrs,
{
  *ups_list = client.get_ups_list().await?;
  debug!("Refreshing the all available Ups devices.");

  for ups in ups_list {
    let cmd = client.get_cmd_list(&ups.name).await?;
    let vars = client.get_var_list(&ups.name).await?;

    debug!(message = "Ups full update queued.", ups_name = &ups.name);

    let update_message = UpsUpdateMessage::FullUpdate {
      commands: cmd.into_iter().map(|Cmd { name: _, cmd }| cmd).collect(),
      name: ups.name.clone(),
      desc: ups.desc.clone(),
      variables: vars.into_iter().map(|Var { name: _, var }| var).collect(),
    };

    channel
      .send(update_message)
      .await
      .map_err(|_| PollServiceError::ChannelError)?;
  }

  Ok(())
}

#[inline]
async fn poll_ups_partial<A>(
  ups_list: &[Ups],
  client: &mut UpsClient<A>,
  channel: &Sender<UpsUpdateMessage>,
) -> Result<(), PollServiceError>
where
  A: ToSocketAddrs,
{
  for ups in ups_list {
    let ups_status = client.get_var(&ups.name, VAR_UPS_STATUS).await?;
    let update_message = UpsUpdateMessage::PartialUpdate {
      name: ups.name.clone(),
      variable: ups_status.var,
    };

    debug!(message = "Ups partial update queued.", ups_name = &ups.name);

    channel
      .send(update_message)
      .await
      .map_err(|_| PollServiceError::ChannelError)?;
  }

  Ok(())
}
