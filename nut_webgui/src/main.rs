use crate::{
  auth::{
    AUTH_COOKIE_DURATION,
    permission::Permissions,
    user_store::{UserProfile, UserStore},
  },
  background_service::BackgroundServiceRunner,
  config::{
    AuthConfig, ServerConfig, UpsdConfig, cfg_arg::ServerCliArgs, cfg_env::ServerEnvArgs,
    cfg_fallback::FallbackArgs, cfg_toml::ServerTomlArgs, cfg_user::UsersConfigFile,
    error::ConfigError,
  },
  event::channel::EventChannel,
  http::{
    HttpServer,
    event_api::message_broadcast::{MessageBroadcast, MessageBroadcastService},
  },
  skip_tls_verifier::SkipTlsVerifier,
  state::{DaemonState, ServerState, UpsdNamespace, UpsdState},
  sync::{
    sync_desc::DescriptionSyncService, sync_device::DeviceSyncService,
    sync_status::StatusSyncService,
  },
};
use nut_webgui_upsmc::{
  client::{NutPoolClient, NutPoolClientBuilder},
  rustls::{ClientConfig, pki_types::ServerName},
};
use rustls_platform_verifier::BuilderVerifierExt;
use std::{collections::HashMap, panic, process::ExitCode, sync::Arc, time::Duration};
use tokio::{
  net::TcpListener,
  select,
  signal::{self, unix::SignalKind},
  sync::RwLock,
};
use tracing::{debug, error, info, level_filters::LevelFilter, warn};
use tracing_subscriber::{
  Registry,
  prelude::*,
  reload::{self, Handle},
};

mod auth;
mod background_service;
mod config;
mod event;
mod http;
mod skip_tls_verifier;
mod state;
mod sync;

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
use mimalloc::MiMalloc;

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> ExitCode {
  panic::set_hook(Box::new(|info| {
    eprintln!("thread panic, details = {}", info);
  }));

  let (filter, handle) = reload::Layer::new(LevelFilter::INFO);

  tracing_subscriber::registry()
    .with(filter)
    .with(tracing_subscriber::fmt::Layer::default())
    .init();

  match nut_webgui(handle) {
    Ok(()) => ExitCode::SUCCESS,
    Err(err) => {
      error!("{}", err);
      ExitCode::FAILURE
    }
  }
}

#[inline]
fn nut_webgui(
  logger_handle: Handle<LevelFilter, Registry>,
) -> Result<(), Box<dyn core::error::Error>> {
  let config = match load_configs() {
    Err(ConfigError::Arguments(e)) => e.exit(),
    v => v,
  }?;

  if config.log_level != LevelFilter::INFO {
    logger_handle.modify(|filter| *filter = config.log_level)?;
  }

  let mut rt = tokio::runtime::Builder::new_multi_thread();
  rt.enable_all();

  if let Some(worker) = config.http_server.worker_count {
    rt.worker_threads(worker.get());
  }

  rt.build()?.block_on(start_server(config))?;

  Ok(())
}

async fn start_server(config: ServerConfig) -> Result<(), Box<dyn core::error::Error>> {
  let mut sigterm = signal::unix::signal(SignalKind::terminate())?;
  let mut sigint = signal::unix::signal(SignalKind::interrupt())?;
  let mut sigquit = signal::unix::signal(SignalKind::quit())?;
  let close_signal = async move {
    select! {
    _ = sigterm.recv() => { info!("SIGTERM signal received."); }
    _ = sigquit.recv() => { info!("SIGQUIT signal received."); }
    _ = sigint.recv() => { info!("SIGINT signal received."); }
    };
  };

  let config = Arc::new(config);

  debug!(message = "server configuration initialized", config = ?config);

  let listener = TcpListener::bind(config.http_server.get_listen_addr())
    .await
    .inspect_err(|err| {
      error!(
        message = "cannot bind tcp socket to listen",
        reason = %err,
        listen_port = config.http_server.port
      );
    })?;

  let event_channel = EventChannel::new(256);
  let message_broadcast = MessageBroadcast::new(256);
  let auth_user_store = create_user_store(&config)?;
  let mut upsd_servers = HashMap::new();

  for (name, upsd_cfg) in config.upsd.iter() {
    let namespace = UpsdNamespace::from(name.as_ref());
    let upsd_state = UpsdState {
      config: upsd_cfg.clone(),
      daemon_state: RwLock::new(DaemonState::new()),
      connection_pool: create_pool(upsd_cfg)?,
      namespace: namespace.clone(),
    };

    upsd_servers.insert(namespace, Arc::new(upsd_state));
  }

  let server_state = Arc::new(ServerState {
    auth_user_store,
    config,
    message_broadcast: message_broadcast.clone(),
    shared_desc: RwLock::new(HashMap::new()),
    upsd_servers,
  });

  let mut bg_services = BackgroundServiceRunner::new()
    .with_max_timeout(Duration::from_secs(10))
    .add_service(DescriptionSyncService::new(
      event_channel.clone(),
      server_state.clone(),
    ))
    .add_service(MessageBroadcastService::new(
      event_channel.clone(),
      message_broadcast,
    ));

  for (name, upsd_state) in server_state.upsd_servers.iter() {
    debug!(
      message = "adding background services for upsd config",
      namespace = name.as_ref()
    );

    let device_sync = DeviceSyncService::new(event_channel.clone(), upsd_state.clone());
    let status_sync = StatusSyncService::new(event_channel.clone(), upsd_state.clone());

    bg_services = bg_services
      .add_service(device_sync)
      .add_service(status_sync);
  }

  debug!(message = "starting background services");
  let service_runner = bg_services.start();
  let http_server = HttpServer::new(server_state.clone());

  http_server
    .serve(listener, close_signal)
    .await
    .inspect_err(|err| {
      error!(message = "http server failed", reason = %err);
    })?;

  info!(message = "http server is closed");
  info!(message = "shutting down background services");

  if let Err(err) = service_runner.stop().await {
    warn!(
      message = "some services are not shutdown properly",
      reason = %err
    );
  }

  info!("closing upsd client connections");

  if let Some(state) = Arc::into_inner(server_state) {
    for upsd in state.upsd_servers.into_values() {
      if let Some(upsd_state) = Arc::into_inner(upsd) {
        upsd_state.connection_pool.close().await;
      }
    }
  }

  Ok(())
}

#[inline]
fn load_configs() -> Result<ServerConfig, ConfigError> {
  let cli_args = ServerCliArgs::load()?;
  let env_args = if cli_args.allow_env {
    ServerEnvArgs::load()?
  } else {
    ServerEnvArgs::default()
  };

  let toml_path = cli_args
    .config_file
    .as_ref()
    .or(env_args.config_file.as_ref());

  let toml_args = if let Some(path) = toml_path {
    ServerTomlArgs::load(path)?
  } else {
    ServerTomlArgs::default()
  };

  let config = ServerConfig::new()
    .layer(toml_args)
    .layer(env_args)
    .layer(cli_args)
    .layer(FallbackArgs);

  Ok(config)
}

fn create_pool(
  config: &UpsdConfig,
) -> Result<NutPoolClient, Box<dyn core::error::Error + 'static>> {
  let tls_client_conf = match config.tls_mode {
    config::tls_mode::TlsMode::Disable => None,
    config::tls_mode::TlsMode::Strict => Some(
      ClientConfig::builder()
        .with_platform_verifier()?
        .with_no_client_auth(),
    ),
    config::tls_mode::TlsMode::SkipVerify => {
      let mut config = ClientConfig::builder()
        .with_platform_verifier()?
        .with_no_client_auth();

      config
        .dangerous()
        .set_certificate_verifier(Arc::new(SkipTlsVerifier));

      Some(config)
    }
  };

  let mut builder = NutPoolClientBuilder::new(config.get_socket_addr().into())
    .with_timeout(Duration::from_secs(config.poll_freq))
    .with_limit(config.max_conn);

  if let Some(tls_config) = tls_client_conf {
    let server_name = ServerName::try_from(config.addr.as_ref().to_owned())?;
    builder = builder.with_tls(server_name, Arc::new(tls_config));
  }

  Ok(builder.build())
}

fn create_user_store(
  config: &ServerConfig,
) -> Result<Option<Arc<UserStore>>, Box<dyn core::error::Error + 'static>> {
  match &config.auth {
    Some(AuthConfig { users_file }) => {
      let users_file = UsersConfigFile::load(users_file)?;
      let mut builder = UserStore::builder().with_session_duration(AUTH_COOKIE_DURATION);

      for (username, user_config) in users_file.users.into_iter() {
        _ = builder.add_user(
          UserProfile {
            username,
            permissions: user_config.permissions.unwrap_or(Permissions::default()),
          },
          user_config.password.as_bytes(),
        );
      }
      let user_store = Arc::new(builder.build());

      Ok(Some(user_store))
    }
    None => Ok(None),
  }
}
