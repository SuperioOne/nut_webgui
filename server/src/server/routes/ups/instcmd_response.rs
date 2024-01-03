use std::net::SocketAddr;
use std::sync::Arc;
use axum::extract::{Path, State};
use axum::Form;
use axum_core::response::IntoResponse;
use tracing::{error, info};
use crate::server::notifications::{Notification, NotificationTemplate};
use crate::server::routes::ups::CommandRequest;
use crate::server::ServerState;
use crate::upsd_client::client::{UpsAuthClient};
use crate::upsd_client::errors::NutClientErrors;

pub(super) async fn instcmd_response(State(state): State<Arc<ServerState>>, Path(ups_name): Path<String>, Form(request): Form<CommandRequest>) -> impl IntoResponse {
  let template: NotificationTemplate = {
    if let (Some(upsd_user), Some(upsd_pass)) = (&state.upsd_config.user, &state.upsd_config.pass) {
      match call_instcmd(&state.upsd_config.addr, upsd_user, upsd_pass, &ups_name, &request.command).await {
        Ok(_) => {
          NotificationTemplate::new(
            format!("Command '{0}' successfully executed for UPS '{1}'.", &request.command, &ups_name),
            Notification::Success,
            None,
          )
        }
        Err(err) => {
          error!("INSTCMD call failed for '{0}'. {1:?}",ups_name, err);
          NotificationTemplate::new(
            format!("INSTCMD call failed, {:?}", err),
            Notification::Error,
            None,
          )
        }
      }
    } else {
      NotificationTemplate::new(
        "No username or password configured for UPS daemon. Server is in read-only mode.".into(),
        Notification::Info,
        None,
      )
    }
  };

  template.into_response()
}

async fn call_instcmd(upsd_addr: &str, upsd_user: &str, upsd_pass: &str, ups_name: &str, cmd: &str) -> Result<(), NutClientErrors> {
  let mut client = UpsAuthClient::create(upsd_addr, upsd_user, upsd_pass).await?;
  _ = client.send_instcmd(ups_name, cmd).await?;
  info!("INSTCMD '{0}' called for UPS '{1}'", cmd, ups_name);

  Ok(())
}