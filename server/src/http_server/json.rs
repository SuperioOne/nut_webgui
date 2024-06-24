use crate::http_server::{ServerState, UpsdConfig};
use crate::ups_mem_store::UpsEntry;
use crate::upsd_client::client::UpsAuthClient;
use crate::upsd_client::errors::NutClientErrors;
use crate::upsd_client::ups_variables::UpsVariable;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_core::response::{IntoResponse, Response};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::info;

impl Serialize for UpsEntry {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut obj = serializer.serialize_struct("UpsEntry", 4)?;
    let mut vars: BTreeMap<String, &UpsVariable> = BTreeMap::new();

    for variable in self.variables.iter() {
      vars.insert(variable.name(), variable);
    }

    obj.serialize_field("name", self.name.as_ref())?;
    obj.serialize_field("desc", self.desc.as_ref())?;
    obj.serialize_field("vars", &vars)?;
    obj.serialize_field("cmds", &self.commands)?;

    obj.end()
  }
}

#[derive(Serialize)]
pub struct ErrorMessage {
  message: String,
  reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CommandBody {
  cmd: String,
}

impl IntoResponse for NutClientErrors {
  fn into_response(self) -> Response {
    match self {
      NutClientErrors::EmptyResponse => (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorMessage {
          message: String::from("UPS Daemon response is empty"),
          reason: None,
        }),
      ),
      NutClientErrors::IOError(error) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorMessage {
          message: String::from("UPS Daemon IO error occurred."),
          reason: Some(error.to_string()),
        }),
      ),
      NutClientErrors::ParseError(error) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorMessage {
          message: String::from("UPS Daemon response is empty"),
          reason: Some(error),
        }),
      ),
      NutClientErrors::ProtocolError(error) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorMessage {
          message: String::from("UPS Daemon response is empty"),
          reason: Some(error.to_string()),
        }),
      ),
    }
    .into_response()
  }
}

pub async fn get_ups_by_name(
  State(state): State<Arc<ServerState>>,
  Path(ups_name): Path<String>,
) -> impl IntoResponse {
  let store = state.store.read().await;

  if let Some(ups) = store.get(&ups_name) {
    Json(ups).into_response()
  } else {
    StatusCode::NOT_FOUND.into_response()
  }
}

pub async fn get_ups_list(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
  let store = state.store.read().await;
  let ups_list: Vec<&UpsEntry> = store.into_iter().map(|(_, entry)| entry).collect();
  Json(ups_list).into_response()
}

pub async fn post_command(
  State(state): State<Arc<ServerState>>,
  Path(ups_name): Path<String>,
  Json(body): Json<CommandBody>,
) -> Result<impl IntoResponse, NutClientErrors> {
  let store = state.store.read().await;

  if store.get(&ups_name).is_some() {
    match &state.upsd_config {
      UpsdConfig {
        addr,
        pass: Some(password),
        user: Some(username),
      } => {
        let mut client = UpsAuthClient::create(addr, username, password).await?;
        client.send_instcmd(&ups_name, &body.cmd).await?;
        info!("INSTCMD '{0}' called for UPS '{1}'", &body.cmd, ups_name);

        Ok(StatusCode::ACCEPTED.into_response())
      }
      _ => Ok(
        (
          StatusCode::UNAUTHORIZED,
          Json(ErrorMessage {
            message: String::from("Insufficient UPSD configuration."),
            reason: Some(String::from(
              "CMD request requires valid username and password to be configured.",
            )),
          }),
        )
          .into_response(),
      ),
    }
  } else {
    Ok(StatusCode::NOT_FOUND.into_response())
  }
}
