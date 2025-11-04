pub mod fsd;
pub mod instcmd;
pub mod namespace;
pub mod not_found;
pub mod rw;
pub mod ups;
pub mod ups_list;

macro_rules! request_auth_client {
  ($upsd_state:expr) => {
    match &$upsd_state.config {
      $crate::config::UpsdConfig {
        pass: Some(pass),
        user: Some(user),
        ..
      } => match $upsd_state.connection_pool.get_client().await {
        Ok(client) => client
          .authenticate(user.as_ref(), pass.as_ref())
          .await
          .map_err(|err| {
            $crate::http::json_api::problem_detail::ProblemDetail::new(
              "Unable to authenticate",
              axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .with_detail(err.to_string())
          }),
        Err(e) => Err(
          $crate::http::json_api::problem_detail::ProblemDetail::new(
            "Unable to get UPSD client",
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
          )
          .with_detail(e.to_string()),
        ),
      },
      _ => Err(
        $crate::http::json_api::problem_detail::ProblemDetail::new(
          "Insufficient UPSD configuration",
          axum::http::StatusCode::FORBIDDEN,
        )
        .with_detail("Operation requires valid username and password to be configured for the UPSD connection.".into()),
      ),
    }
  };
}

macro_rules! extract_upsd {
  ($state:expr, $namespace:expr) => {
    match $state.upsd_servers.get(&$namespace) {
      Some(upsd) => Ok(upsd),
      None => Err(ProblemDetail::new(
        "Upsd namespace does not exists",
        axum::http::StatusCode::NOT_FOUND,
      )),
    }
  };
}

pub(super) use extract_upsd;
pub(super) use request_auth_client;
