use crate::{
  auth::{permission::Permissions, user_session::UserSession},
  config::ServerConfig,
};
use askama::Template;
use std::{any::Any, collections::HashMap};

pub trait RenderWithConfig: Template + Sized {
  fn render_with_config(
    self,
    config: &ServerConfig,
    user: Option<&UserSession>,
  ) -> Result<String, askama::Error>;
}

impl<T> RenderWithConfig for T
where
  T: Template,
{
  fn render_with_config(
    self,
    config: &ServerConfig,
    user: Option<&UserSession>,
  ) -> Result<String, askama::Error> {
    let mut permissions = Permissions::all();
    let mut values: HashMap<&'static str, &dyn Any> = HashMap::new();

    values.insert("HTTP_SERVER__BASE_PATH", &config.http_server.base_path);

    if let Some(theme) = &config.default_theme {
      values.insert("DEFAULT_THEME", theme);
    }

    if let Some(profile) = user {
      values.insert("USER_NAME", profile.get_username());
      permissions = profile.get_permissions();
    }

    values.insert("USER_PERMISSION", &permissions);
    self.render_with_values(&values)
  }
}
