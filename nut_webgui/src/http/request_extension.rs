use crate::auth::{access_token::AccessToken, user_session::UserSession};

#[derive(Clone)]
pub enum AuthExtension {
  API(AccessToken),
  User(UserSession),
}
