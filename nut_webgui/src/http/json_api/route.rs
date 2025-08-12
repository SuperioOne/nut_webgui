pub mod fsd;
pub mod instcmd;
pub mod not_found;
pub mod rw;
pub mod ups;
pub mod ups_list;

macro_rules! request_auth_client {
  ($route_state:expr) => {
    match &$route_state.config.upsd {
      $crate::config::UpsdConfig {
        pass: Some(pass),
        user: Some(user),
        ..
      } => match $route_state.connection_pool.get_client().await {
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
          "Insufficient upsd configuration",
          axum::http::StatusCode::UNAUTHORIZED,
        )
        .with_detail("Operation requires valid username and password to be configured.".into()),
      ),
    }
  };
}

pub(super) use request_auth_client;
