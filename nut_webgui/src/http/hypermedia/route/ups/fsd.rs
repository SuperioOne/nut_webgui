use crate::{
  auth::user_session::UserSession,
  config::UpsdConfig,
  htmx_redirect,
  http::{
    RouterState,
    hypermedia::{
      error::ErrorPage, notification::NotificationTemplate, semantic_type::SemanticType,
      utils::RenderWithConfig,
    },
  },
};
use axum::{
  Extension,
  extract::{Path, State},
  http::StatusCode,
  response::{Html, IntoResponse, Response},
};
use nut_webgui_upsmc::UpsName;
use tracing::{error, info};

pub async fn post(
  State(rs): State<RouterState>,
  Path(ups_name): Path<UpsName>,
  session: Option<Extension<UserSession>>,
) -> Result<Response, ErrorPage> {
  {
    let state = rs.state.read().await;

    if state.devices.get(&ups_name).is_none() {
      return Ok(
        htmx_redirect!(
          StatusCode::NOT_FOUND,
          format!("{}/not-found", rs.config.http_server.base_path)
        )
        .into_response(),
      );
    }
  };

  let session = session.map(|v| v.0);
  let auth_client = match &rs.config.upsd {
    UpsdConfig {
      pass: Some(pass),
      user: Some(user),
      ..
    } => {
      let client = rs.connection_pool.get_client().await?;
      client.authenticate(user, pass).await
    }
    _ => {
      return Ok(
        Html(
          NotificationTemplate::from(
            "No username or password configured for UPS daemon. Server is in read-only mode.",
          )
          .render_with_config(&rs.config, session.as_ref())?,
        )
        .into_response(),
      );
    }
  };

  let fsd_result = match auth_client {
    Ok(mut client) => {
      let result = client.fsd(&ups_name).await;
      _ = client.close().await;
      result
    }
    Err(e) => Err(e),
  };

  let template = match fsd_result {
    Ok(_) => {
      info!(message = "forced-shutdown called successfully", device_name = %ups_name);

      NotificationTemplate::from(format!("FSD flag set on {0}.", &ups_name))
        .set_level(SemanticType::Warning)
    }
    Err(err) => {
      error!(message = "fsd call failed", device_name = %ups_name, reason = %err);

      NotificationTemplate::from(format!("FSD failed, {}", err)).set_level(SemanticType::Error)
    }
  };

  Ok(Html(template.render_with_config(&rs.config, session.as_ref())?).into_response())
}
