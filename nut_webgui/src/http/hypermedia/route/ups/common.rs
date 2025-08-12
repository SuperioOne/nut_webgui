macro_rules! request_auth_client {
  ($route_state:expr) => {
    match &$route_state.config.upsd {
      UpsdConfig {
        pass: Some(pass),
        user: Some(user),
        ..
      } => match $route_state.connection_pool.get_client().await {
        Ok(client) => client
          .authenticate(user.as_ref(), pass.as_ref())
          .await
          .map_err(|err| {
            ProblemDetail::new("Unable to authenticate", StatusCode::INTERNAL_SERVER_ERROR)
              .with_detail(err.to_string())
          }),
        Err(e) => Err(
          ProblemDetail::new(
            "Unable to get UPSD client",
            StatusCode::INTERNAL_SERVER_ERROR,
          )
          .with_detail(e.to_string()),
        ),
      },
      _ => Err(
        ProblemDetail::new("Insufficient upsd configuration", StatusCode::UNAUTHORIZED)
          .with_detail("Operation requires valid username and password to be configured.".into()),
      ),
    }
  };
}
