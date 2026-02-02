use crate::{
  auth::{
    AUTH_COOKIE_NAME, password_str::PasswordStr, token_signer::TokenSigner,
    user_session::UserSession, username::Username,
  },
  http::hypermedia::{error::ErrorPage, util::RenderWithConfig},
  state::ServerState,
};
use askama::Template;
use axum::{
  Extension, Form,
  extract::{State, rejection::FormRejection},
  http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header},
  response::{Html, IntoResponse, Redirect, Response},
};
use base64::{Engine, prelude::BASE64_STANDARD};
use cookie::{Cookie, SameSite};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Template, Default)]
#[template(path = "login/+page.html", ext= "html", blocks = ["login_form"])]
struct LoginTemplate<'a> {
  error_message: Option<&'a str>,
}

pub async fn get(
  state: State<Arc<ServerState>>,
  session: Option<Extension<UserSession>>,
) -> Result<Response, ErrorPage> {
  let redirect_path = format!("{}/", state.config.http_server.base_path);

  if let Some(session) = session.map(|v| v.0)
    && session.is_active()
    && state
      .auth_user_store
      .as_ref()
      .is_some_and(|s| s.contains_user(session.get_username()))
  {
    Ok(Redirect::to(&redirect_path).into_response())
  } else {
    let template = LoginTemplate::default().render_with_config(&state.config, None)?;
    Ok(Html(template).into_response())
  }
}

#[derive(Deserialize)]
pub struct LoginForm {
  username: Username,
  password: PasswordStr,
}

pub async fn post(
  state: State<Arc<ServerState>>,
  session: Option<Extension<UserSession>>,
  login_form: Result<Form<LoginForm>, FormRejection>,
) -> Result<Response, ErrorPage> {
  let login_form = match login_form {
    Ok(form) => form.0,
    Err(err) => {
      let err_message = err.to_string();
      let template = LoginTemplate {
        error_message: Some(&err_message),
      };

      return Ok(
        Html(
          template
            .as_login_form()
            .render_with_config(&state.config, None)?,
        )
        .into_response(),
      );
    }
  };

  let redirect_path = format!("{}/", state.config.http_server.base_path);
  if let Some(session) = session.map(|v| v.0)
    && session.is_active()
    && state
      .auth_user_store
      .as_ref()
      .is_some_and(|s| s.contains_user(session.get_username()))
  {
    Ok(Redirect::to(&redirect_path).into_response())
  } else {
    match state.auth_user_store.as_ref() {
      Some(store) => match store.login_user(&login_form.username, &login_form.password) {
        Ok(session) => {
          let ttl = session.ttl();
          let signed_bytes =
            TokenSigner::new(state.config.server_key.as_bytes()).sign_token(&session);

          let cookie = Cookie::build((AUTH_COOKIE_NAME, BASE64_STANDARD.encode(signed_bytes)))
            .http_only(true)
            .same_site(SameSite::Strict)
            .path("/")
            .max_age(cookie::time::Duration::seconds(ttl.as_secs() as i64))
            .build();

          let mut headers = HeaderMap::new();
          headers.insert(
            header::SET_COOKIE,
            HeaderValue::from_str(&cookie.to_string())?,
          );
          headers.insert(
            HeaderName::from_static("hx-redirect"),
            HeaderValue::from_str(&redirect_path)?,
          );

          Ok((StatusCode::OK, headers).into_response())
        }
        Err(_) => {
          let template = LoginTemplate {
            error_message: Some("Invalid credentials, check your username and password!"),
          };

          Ok(
            Html(
              template
                .as_login_form()
                .render_with_config(&state.config, None)?,
            )
            .into_response(),
          )
        }
      },
      None => Err(ErrorPage::new(
        "login routes are enabled but user store is missing".to_owned(),
      )),
    }
  }
}
