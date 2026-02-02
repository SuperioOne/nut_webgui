use super::error::UserTomlError;
use crate::auth::{password_str::PasswordStr, permission::Permissions, username::Username};
use serde::{Deserialize, de::Visitor};
use std::{collections::HashMap, fs::File, io::Read, path::Path};

#[derive(Deserialize)]
pub struct UserConfig {
  pub password: PasswordStr,
  pub permissions: Option<Permissions>,
}

pub struct UsersConfigFile {
  pub users: HashMap<Username, UserConfig>,
}

impl UsersConfigFile {
  pub fn load<P>(path: P) -> Result<Self, UserTomlError>
  where
    P: AsRef<Path>,
  {
    let mut fd = File::open(path)?;
    let mut buffer = String::new();
    _ = fd.read_to_string(&mut buffer)?;

    let deserializer = toml::Deserializer::parse(&buffer)?;
    let users_config = Self::deserialize(deserializer)?;

    unsafe {
      buffer.as_bytes_mut().fill(0);
    }

    Ok(users_config)
  }
}

struct UsersConfigVisitor;

impl<'de> Visitor<'de> for UsersConfigVisitor {
  type Value = UsersConfigFile;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("UserName=UserConfig map")
  }

  fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::MapAccess<'de>,
  {
    let mut config = UsersConfigFile {
      users: HashMap::new(),
    };

    while let Some((k, v)) = map.next_entry::<Username, UserConfig>()? {
      _ = config.users.insert(k, v);
    }

    Ok(config)
  }
}

impl<'de> Deserialize<'de> for UsersConfigFile {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_map(UsersConfigVisitor)
  }
}
