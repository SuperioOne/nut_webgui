use crate::{
  auth::{
    access_token::AccessToken, permission::Permissions, signed_token::SignedToken,
    user_session::UserSession,
  },
  htmx_redirect, htmx_swap,
  http::{
    RouterState,
    hypermedia::{
      error::ErrorPage, notification::NotificationTemplate, semantic_type::SemanticType,
      utils::RenderWithConfig,
    },
  },
};
use askama::Template;
use axum::{
  Extension, Form,
  extract::{State, rejection::FormRejection},
  http::StatusCode,
  response::{Html, IntoResponse, Response},
};
use base64::{Engine, prelude::BASE64_STANDARD};
use serde::Deserialize;
use std::{num::NonZeroU64, time::Duration};
use tracing::{info, warn};

#[derive(Template)]
#[template(path = "api_key/+page.html", ext= "html", blocks = ["api_key_form"])]
struct ApiKeyFormTemplate<'a> {
  error_message: Option<&'a str>,
}

#[derive(Template)]
#[template(path = "api_key/success.html", ext = "html")]
struct ApiKeySuccessTemplate<'a> {
  key: &'a str,
}

pub async fn get(
  rs: State<RouterState>,
  session: Extension<UserSession>,
) -> Result<Response, ErrorPage> {
  let template = ApiKeyFormTemplate {
    error_message: None,
  };

  let response = Html(template.render_with_config(&rs.config, Some(&session.0))?).into_response();

  Ok(response)
}
#[derive(Deserialize, Debug)]
pub struct ApiKeyForm {
  permissions: Permissions,
  duration: NonZeroU64,
}

pub async fn post(
  rs: State<RouterState>,
  session: Extension<UserSession>,
  form: Result<Form<ApiKeyForm>, FormRejection>,
) -> Result<Response, ErrorPage> {
  let session = session.0;

  let response = match form {
    Ok(key_request) => {
      let ApiKeyForm {
        permissions,
        duration,
      } = key_request.0;

      if rs
        .auth_user_store
        .as_ref()
        .is_some_and(|s| s.contains_user(session.get_username()))
      {
        if session.has_permission(permissions) {
          let access_token = AccessToken::builder()
            .with_permissions(permissions)
            .with_valid_until(Duration::from_millis(duration.get()))
            .build();

          let signed_bytes = SignedToken::<AccessToken>::new(rs.config.server_key.as_bytes())
            .sign_token(&access_token);

          info!(message = "new api key generated", issuer = %session.get_username());

          let encoded = BASE64_STANDARD.encode(signed_bytes);
          let template = ApiKeySuccessTemplate { key: &encoded };

          Html(template.render_with_config(&rs.config, Some(&session))?).into_response()
        } else {
          htmx_swap!(
            Html(
              NotificationTemplate::new("User doesn't have the necessary permissions.")
                .set_level(SemanticType::Error)
                .render_with_config(&rs.config, Some(&session))?,
            ),
            "none"
          )
        }
      } else {
        warn!(message = "key generation aborted, user no longer exists in the users list", user = %session.get_username());

        htmx_redirect!(
          StatusCode::OK,
          format!("{}/", rs.config.http_server.base_path.as_str())
        )
        .into_response()
      }
    }
    Err(err) => {
      let details = err.body_text();
      let template = ApiKeyFormTemplate {
        error_message: Some(&details),
      };

      Html(
        template
          .as_api_key_form()
          .render_with_config(&rs.config, Some(&session))?,
      )
      .into_response()
    }
  };

  Ok(response)
}
