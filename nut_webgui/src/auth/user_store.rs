use crate::auth::{
  access_token::AccessToken, password_str::PasswordStr, permission::Permissions,
  user_session::UserSession, username::Username,
};
use core::borrow::Borrow;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};
use scrypt::Params;
use std::{collections::HashMap, time::Duration};

pub struct User {
  profile: UserProfile,
  salt: Box<[u8; 32]>,
  password_hash: Box<[u8; 32]>,
}

pub struct UserProfile {
  pub permissions: Permissions,
  pub username: Username,
}

#[derive(Default)]
pub struct UserStore {
  inner: HashMap<Username, User>,
  session_duration: Duration,
  hash_params: Params,
}

#[derive(Debug)]
pub enum SessionError {
  InvalidLogin,
  InvalidOutputLength,
}

#[inline(never)]
fn cmp_hash(lhs: &[u8; 32], rhs: &[u8; 32]) -> bool {
  let lhs_ptr: *const u128 = lhs.as_ptr().cast();
  let rhs_ptr: *const u128 = rhs.as_ptr().cast();
  let mut result: u128 = 0;

  for i in 0..2 {
    result |= unsafe { lhs_ptr.add(i).read_unaligned() ^ rhs_ptr.add(i).read_unaligned() };
  }

  result == 0
}

impl UserStore {
  #[inline]
  pub fn get_profile(&self, username: &Username) -> Option<&UserProfile> {
    self.inner.get(username).map(|v| &v.profile)
  }

  #[inline]
  pub fn builder() -> UserStoreBuilder {
    UserStoreBuilder::new()
  }

  pub fn login_user(
    &self,
    username: &Username,
    password: &PasswordStr,
  ) -> Result<UserSession, SessionError> {
    match self.inner.get(username) {
      Some(user) => {
        let mut output = [0u8; 32];

        scrypt::scrypt(
          password.as_bytes(),
          user.salt.as_slice(),
          &self.hash_params,
          &mut output,
        )
        .map_err(|_| SessionError::InvalidOutputLength)?;

        if cmp_hash(&output, &user.password_hash) {
          let access_token = AccessToken::builder()
            .with_permissions(user.profile.permissions)
            .with_valid_until(self.session_duration)
            .build();

          Ok(UserSession::new(username.clone(), access_token))
        } else {
          Err(SessionError::InvalidLogin)
        }
      }
      None => Err(SessionError::InvalidLogin),
    }
  }

  pub fn renew_session(&self, username: &Username) -> Result<UserSession, SessionError> {
    match self.inner.get(username) {
      Some(user) => {
        let access_token = AccessToken::builder()
          .with_permissions(user.profile.permissions)
          .with_valid_until(self.session_duration)
          .build();

        Ok(UserSession::new(username.clone(), access_token))
      }
      None => Err(SessionError::InvalidLogin),
    }
  }

  #[inline]
  pub fn contains_user<K>(&self, username: K) -> bool
  where
    K: Borrow<Username>,
  {
    self.inner.contains_key(username.borrow())
  }
}

pub struct UserStoreBuilder {
  users: HashMap<Username, User>,
  session_duration: Option<Duration>,
  params: Params,
}

fn random_salt() -> Box<[u8; 32]> {
  let mut rng = ChaCha20Rng::from_os_rng();
  let rand: [u64; 4] = [
    rng.next_u64(),
    rng.next_u64(),
    rng.next_u64(),
    rng.next_u64(),
  ];

  let rand_bytes: [u8; 32] = unsafe { core::mem::transmute(rand) };

  Box::from(rand_bytes)
}

impl UserStoreBuilder {
  pub fn new() -> Self {
    let params = Params::new(10, 10, 2, 32).expect("scrypt parameters should've never fail");

    Self {
      users: HashMap::new(),
      session_duration: None,
      params,
    }
  }

  pub fn add_user(&mut self, profile: UserProfile, password: &[u8]) -> Option<User> {
    let key = profile.username.clone();
    let salt = random_salt();
    let mut hash = [0u8; 32];

    scrypt::scrypt(password, salt.as_slice(), &self.params, &mut hash)
      .expect("invalid password output length configuration");

    let user = User {
      profile,
      salt,
      password_hash: Box::from(hash),
    };

    self.users.insert(key, user)
  }

  #[inline]
  pub fn with_session_duration(mut self, duration: Duration) -> Self {
    self.session_duration = Some(duration);
    self
  }

  pub fn build(self) -> UserStore {
    const WEEK_IN_SECS: u64 = 604800;

    UserStore {
      inner: self.users,
      hash_params: self.params,
      session_duration: self
        .session_duration
        .unwrap_or(Duration::from_secs(WEEK_IN_SECS)),
    }
  }
}

impl core::fmt::Display for SessionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      SessionError::InvalidLogin => f.write_str("username is not present in user list"),
      SessionError::InvalidOutputLength => f.write_str("hashing output length is invalid"),
    }
  }
}

impl core::error::Error for SessionError {}
