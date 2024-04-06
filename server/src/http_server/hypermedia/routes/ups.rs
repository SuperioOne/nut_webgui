use crate::http_server::hypermedia::notifications::{Notification, NotificationTemplate};
use crate::http_server::ServerState;
use crate::upsd_client::client::UpsAuthClient;
use crate::upsd_client::errors::NutClientErrors;
use axum::extract::{Path, Query, State};
use axum::Form;
use axum_core::response::IntoResponse;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

mod html_responses;

#[derive(Deserialize)]
pub struct UpsQuery {
  section: Option<String>,
}

#[derive(Deserialize)]
pub struct CommandRequest {
  command: String,
}

pub async fn get(
  path: Path<String>,
  query: Query<UpsQuery>,
  state: State<Arc<ServerState>>,
) -> impl IntoResponse {
  if let Some(section) = &query.section {
    match section.as_ref() {
      "var" => {
        return html_responses::partial_ups_vars(path, state)
          .await
          .into_response()
      }
      "info" => {
        return html_responses::partial_ups_info(path, state)
          .await
          .into_response()
      }
      _ => {}
    };
  }

  html_responses::page_response(path, state)
    .await
    .into_response()
}

pub async fn post_command(
  State(state): State<Arc<ServerState>>,
  Path(ups_name): Path<String>,
  Form(request): Form<CommandRequest>,
) -> impl IntoResponse {
  let template: NotificationTemplate = {
    if let (Some(user), Some(pass)) = (&state.upsd_config.user, &state.upsd_config.pass) {
      match call_instcmd(
        &state.upsd_config.addr,
        user,
        pass,
        &ups_name,
        &request.command,
      )
      .await
      {
        Ok(_) => NotificationTemplate::new(
          format!(
            "Command '{0}' successfully executed for UPS '{1}'.",
            &request.command, &ups_name
          ),
          Notification::Success,
          None,
        ),
        Err(err) => {
          error!("INSTCMD call failed for '{0}'. {1:?}", ups_name, err);
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

async fn call_instcmd(
  addr: &str,
  user: &str,
  pass: &str,
  ups_name: &str,
  cmd: &str,
) -> Result<(), NutClientErrors> {
  let mut client = UpsAuthClient::create(addr, user, pass).await?;
  client.send_instcmd(ups_name, cmd).await?;
  info!("INSTCMD '{0}' called for UPS '{1}'", cmd, ups_name);

  Ok(())
}
