use super::{RouterState, problem_detail::ProblemDetail};
use crate::{
  config::UpsdConfig,
  state::{CommandsCacheEntry, DescriptionKey},
};
use axum::http::StatusCode;
use nut_webgui_upsmc::errors::{ErrorKind, ProtocolError};
use nut_webgui_upsmc::{InstCmd, UpsName, clients::NutAuthClient};
use std::time::{Duration, Instant};

pub async fn get_cached_commands(rs: &RouterState, ups_name: &UpsName) -> (Vec<InstCmd>, bool) {
  let ttl = Duration::from_secs(rs.config.commands_ttl);
  let now = Instant::now();
  let state = rs.state.read().await;
  if let Some(entry) = state.commands_cache.get(ups_name) {
    let stale = now.duration_since(entry.fetched_at) >= ttl;
    (entry.commands.clone(), stale)
  } else {
    (Vec::new(), true)
  }
}

pub async fn update_commands(
  rs: &RouterState,
  ups_name: &UpsName,
) -> Result<Vec<InstCmd>, ProblemDetail> {
  let (addr, user, password) = match &rs.config.upsd {
    UpsdConfig {
      pass: Some(pass),
      user: Some(user),
      ..
    } => (
      rs.config.upsd.get_socket_addr(),
      user.as_ref(),
      pass.as_ref(),
    ),
    _ => {
      return Err(
        ProblemDetail::new("Insufficient upsd configuration", StatusCode::UNAUTHORIZED)
          .with_detail("Operation requires valid username and password to be configured.".into()),
      );
    }
  };
  let mut client = match NutAuthClient::connect(addr, user, password).await {
    Ok(c) => c,
    Err(err) => {
      return match err.kind() {
        ErrorKind::ProtocolError {
          inner: ProtocolError::AccessDenied,
        } => Err(ProblemDetail::new(
          "Access denied",
          StatusCode::UNAUTHORIZED,
        )),
        ErrorKind::ProtocolError {
          inner: ProtocolError::UnknownUps,
        } => Err(ProblemDetail::new(
          "Device not found",
          StatusCode::NOT_FOUND,
        )),
        ErrorKind::IOError { .. } | ErrorKind::RequestTimeout => Err(ProblemDetail::new(
          "UPS daemon unreachable",
          StatusCode::BAD_GATEWAY,
        )),
        _ => Err(err.into()),
      };
    }
  };
  let cmds = match client.list_instcmds(ups_name).await {
    Ok(c) => c,
    Err(err) => {
      return match err.kind() {
        ErrorKind::ProtocolError {
          inner: ProtocolError::AccessDenied,
        } => Err(ProblemDetail::new(
          "Access denied",
          StatusCode::UNAUTHORIZED,
        )),
        ErrorKind::ProtocolError {
          inner: ProtocolError::UnknownUps,
        } => Err(ProblemDetail::new(
          "Device not found",
          StatusCode::NOT_FOUND,
        )),
        ErrorKind::IOError { .. } | ErrorKind::RequestTimeout => Err(ProblemDetail::new(
          "UPS daemon unreachable",
          StatusCode::BAD_GATEWAY,
        )),
        _ => Err(err.into()),
      };
    }
  };
  _ = client.close().await;
  let mut state = rs.state.write().await;
  state.commands_cache.insert(
    ups_name.clone(),
    CommandsCacheEntry {
      fetched_at: Instant::now(),
      commands: cmds.clone(),
    },
  );
  if let Some(device) = state.devices.get_mut(ups_name) {
    device.commands = cmds.iter().map(|c| c.id.clone()).collect();
  }
  for c in &cmds {
    state.shared_desc.insert(
      DescriptionKey::from(c.id.clone()),
      Box::from(c.desc.clone()),
    );
  }
  Ok(cmds)
}
