use crate::{
  htmx_redirect,
  http_server::{
    hypermedia::{
      notifications::{Notification, NotificationTemplate},
      ups_info::UpsInfo,
    },
    ServerState,
  },
  ups_mem_store::UpsEntry,
  upsd_client::{client::UpsAuthClient, errors::NutClientErrors},
};
use askama::Template;
use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  response::Redirect,
  Form,
};
use axum_core::response::IntoResponse;
use serde::Deserialize;
use std::{borrow::Borrow, sync::Arc};
use tracing::{error, info};

#[derive(Deserialize)]
pub struct UpsFragmentQuery {
  section: Option<String>,
}

#[derive(Deserialize)]
pub struct CommandRequest {
  command: String,
}

#[derive(Template)]
#[template(path = "ups/ups_info.html", ext = "html")]
struct UpsInfoTemplate {
  ups_info: UpsInfo,
  variables: Vec<(String, String)>,
}

// TODO: Switch to block fragments when askama v0.13 released
#[derive(Template)]
#[template(path = "ups/+page.html", ext = "html", escape = "none")]
struct UpsPageTemplate<'a> {
  title: &'a str,
  ups_info: UpsInfoTemplate,
  commands: &'a [Box<str>],
}

impl<T> From<T> for UpsInfoTemplate
where
  T: Borrow<UpsEntry>,
{
  fn from(value: T) -> Self {
    let variables: Vec<(String, String)> = value
      .borrow()
      .variables
      .iter()
      .map(|e| (e.name(), e.value_as_string()))
      .collect();

    Self {
      ups_info: UpsInfo::from(value),
      variables,
    }
  }
}

async fn page_response(
  Path(ups_name): Path<String>,
  State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
  let ups_name = ups_name.as_str();

  if let Some(ups) = state.store.read().await.get(ups_name) {
    let template = UpsPageTemplate {
      title: ups_name,
      ups_info: UpsInfoTemplate::from(ups),
      commands: &ups.commands,
    };

    template.into_response()
  } else {
    Redirect::permanent("/not-found").into_response()
  }
}

async fn partial_ups_info(
  Path(ups_name): Path<String>,
  State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
  let ups_name = ups_name.as_str();

  if let Some(ups) = state.store.read().await.get(ups_name) {
    UpsInfoTemplate::from(ups).into_response()
  } else {
    htmx_redirect!(StatusCode::NOT_FOUND, "/not-found").into_response()
  }
}

pub async fn get(
  path: Path<String>,
  query: Query<UpsFragmentQuery>,
  state: State<Arc<ServerState>>,
) -> impl IntoResponse {
  if let Some(section) = query.section.as_deref() {
    match section {
      "info" => return partial_ups_info(path, state).await.into_response(),
      _ => {
        // ignore invalid section names
      }
    };
  }

  page_response(path, state).await.into_response()
}

pub async fn post_command(
  State(state): State<Arc<ServerState>>,
  Path(ups_name): Path<String>,
  Form(request): Form<CommandRequest>,
) -> impl IntoResponse {
  let template: NotificationTemplate = {
    if let (Some(user), Some(pass)) = (&state.upsd_config.user, &state.upsd_config.pass) {
      match {
        let addr: &str = &state.upsd_config.addr;
        let ups_name: &str = &ups_name;
        let cmd: &str = &request.command;

        async move {
          let mut client = UpsAuthClient::create(addr, user, pass).await?;
          client.send_instcmd(ups_name, cmd).await?;
          info!("INSTCMD '{0}' called for UPS '{1}'", cmd, ups_name);
          Ok::<(), NutClientErrors>(())
        }
      }
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

  template
}
