use super::common::ProblemDetailsResponse;
use crate::{
  http_server::{ServerConfig, ServerState},
  ups_daemon_state::UpsEntry,
  upsd_client::{client::UpsAuthClient, errors::NutClientErrors, ups_variables::UpsVariable},
};
use axum::{
  extract::{OriginalUri, Path, State},
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use std::{collections::BTreeMap, ops::Deref};
use tracing::info;

impl Serialize for UpsEntry {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut obj = serializer.serialize_struct("UpsEntry", 4)?;
    let mut vars: BTreeMap<&str, &UpsVariable> = BTreeMap::new();

    for variable in self.variables.iter() {
      vars.insert(variable.name(), variable);
    }

    obj.serialize_field("name", &self.name)?;
    obj.serialize_field("desc", &self.desc)?;
    obj.serialize_field("vars", &vars)?;
    obj.serialize_field("cmds", &self.commands)?;

    obj.end()
  }
}

#[derive(Debug, Deserialize)]
pub struct CommandBody {
  cmd: String,
}

pub async fn get_ups_by_name(
  State(state): State<ServerState>,
  uri: OriginalUri,
  Path(ups_name): Path<String>,
) -> Response {
  let upsd_state = state.upsd_state.read().await;

  if let Some(ups) = upsd_state.get_ups(&ups_name) {
    Json(ups).into_response()
  } else {
    ProblemDetailsResponse {
      status: StatusCode::NOT_FOUND,
      instance: Some(uri.to_string()),
      title: "UPS not found",
      detail: None,
    }
    .into_response()
  }
}

pub async fn get_ups_list(State(state): State<ServerState>) -> Response {
  let upsd_state = state.upsd_state.read().await;
  let mut ups_list: Vec<&UpsEntry> = upsd_state.iter().map(|(_, entry)| entry).collect();
  ups_list.sort_unstable_by_key(|v| &v.name);

  Json(ups_list).into_response()
}

pub async fn post_command(
  State(state): State<ServerState>,
  uri: OriginalUri,
  Path(ups_name): Path<String>,
  Json(body): Json<CommandBody>,
) -> Response {
  let upsd_state = state.upsd_state.read().await;

  match upsd_state.get_ups(&ups_name) {
    Some(_) => match state.configs.deref() {
      ServerConfig {
        addr,
        pass: Some(password),
        user: Some(username),
        ..
      } => {
        let command_result: Result<(), NutClientErrors> = async {
          let mut client = UpsAuthClient::create(addr, username, password).await?;
          client.send_instcmd(&ups_name, &body.cmd).await?;
          Ok(())
        }
        .await;

        info!("INSTCMD '{0}' called for UPS '{1}'", &body.cmd, ups_name);

        match command_result {
          Ok(()) => StatusCode::ACCEPTED.into_response(),
          Err(err) => {
            let mut problem = ProblemDetailsResponse::from(err);
            problem.instance = Some(uri.to_string());

            problem.into_response()
          }
        }
      }
      _ => ProblemDetailsResponse {
        status: StatusCode::UNAUTHORIZED,
        instance: Some(uri.to_string()),
        title: "Insufficient Upsd configuration",
        detail: Some(String::from(
          "CMD command requires valid username and password to be configured.",
        )),
      }
      .into_response(),
    },
    None => ProblemDetailsResponse {
      status: StatusCode::NOT_FOUND,
      instance: Some(uri.to_string()),
      title: "UPS not found",
      detail: None,
    }
    .into_response(),
  }
}
