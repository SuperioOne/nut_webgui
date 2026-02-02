use crate::{
  auth::user_session::UserSession,
  config::UpsdConfig,
  http::hypermedia::{
    error::ErrorPage,
    notification::NotificationTemplate,
    semantic_type::SemanticType,
    util::{RenderWithConfig, redirect_not_found},
  },
  state::ServerState,
};
use axum::{
  Extension, Form,
  extract::{Path, State},
  response::{Html, IntoResponse, Response},
};
use nut_webgui_upsmc::{CmdName, UpsName};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, info};

#[derive(Deserialize, Debug)]
pub struct CommandRequest {
  command: CmdName,
}

pub async fn post(
  State(state): State<Arc<ServerState>>,
  Path((namespace, ups_name)): Path<(Box<str>, UpsName)>,
  session: Option<Extension<UserSession>>,
  Form(request): Form<CommandRequest>,
) -> Result<Response, ErrorPage> {
  let upsd = match state.upsd_servers.get(namespace.as_ref()) {
    Some(upsd) => upsd,
    None => return Ok(redirect_not_found!(&state)),
  };

  if let None = upsd.daemon_state.read().await.devices.get(&ups_name) {
    return Ok(redirect_not_found!(&state));
  }

  let session = session.map(|v| v.0);
  let auth_client = match &upsd.config {
    UpsdConfig {
      pass: Some(pass),
      user: Some(user),
      ..
    } => {
      let client = upsd.connection_pool.get_client().await?;
      client.authenticate(user, pass).await
    }
    _ => {
      return Ok(
        Html(
          NotificationTemplate::from(
            "No username or password configured for UPS daemon. Server is in read-only mode.",
          )
          .render_with_config(&state.config, session.as_ref())?,
        )
        .into_response(),
      );
    }
  };

  let cmd_result = match auth_client {
    Ok(mut client) => {
      let result = client.instcmd(&ups_name, &request.command).await;
      _ = client.close().await;
      result
    }
    Err(e) => Err(e),
  };

  let template = match cmd_result {
    Ok(_) => {
      info!(
        message = "instcmd called successfully",
        namespace = %namespace,
        device_name = %ups_name,
        cmd = %request.command
      );

      NotificationTemplate::from(format!(
        "'{0}' successfully executed on {1}.",
        &request.command, &ups_name
      ))
      .set_level(SemanticType::Success)
    }
    Err(err) => {
      error!(
        message = "instcmd call failed",
        namespace = %namespace,
        device_name = %ups_name,
        cmd = %request.command,
        reason = %err
      );

      NotificationTemplate::from(format!("INSTCMD call failed, {}", err))
        .set_level(SemanticType::Error)
    }
  };

  Ok(Html(template.render_with_config(&state.config, session.as_ref())?).into_response())
}
