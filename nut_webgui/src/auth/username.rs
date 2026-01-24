use askama::FastWritable;
use serde::{
  Deserialize,
  de::{self, Visitor},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Username(Box<str>);

impl TryFrom<Box<str>> for Username {
  type Error = EmptyUsername;

  fn try_from(value: Box<str>) -> Result<Self, Self::Error> {
    if value.as_ref().trim().is_empty() {
      Err(EmptyUsername)
    } else {
      Ok(Self(value))
    }
  }
}

impl TryFrom<String> for Username {
  type Error = EmptyUsername;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    if value.trim().is_empty() {
      Err(EmptyUsername)
    } else {
      Ok(Self(value.into_boxed_str()))
    }
  }
}

impl core::fmt::Display for Username {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmptyUsername;

impl core::error::Error for EmptyUsername {}

impl core::fmt::Display for EmptyUsername {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str("username cannot be empty")
  }
}

impl Username {
  pub fn new(username: &str) -> Result<Self, EmptyUsername> {
    if username.trim().is_empty() {
      Err(EmptyUsername)
    } else {
      Ok(Self(Box::from(username)))
    }
  }

  pub fn get_initials(&self) -> &str {
    match self.0.len() {
      0 => "",
      1 => &self.0,
      _ => &self.0[0..2],
    }
  }
}

impl core::ops::Deref for Username {
  type Target = str;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl AsRef<str> for Username {
  #[inline]
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl FastWritable for Username {
  #[inline]
  fn write_into<W: core::fmt::Write + ?Sized>(
    &self,
    dest: &mut W,
    _: &dyn askama::Values,
  ) -> askama::Result<()> {
    dest.write_str(&self.0)?;
    Ok(())
  }
}

struct UserNameVisitor;

impl<'de> Visitor<'de> for UserNameVisitor {
  type Value = Username;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("non-empty username string")
  }

  fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    let username =
      Self::Value::try_from(v).map_err(|_| de::Error::custom("empty username string"))?;

    Ok(username)
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    let username = Username::new(v).map_err(|_| de::Error::custom("empty username string"))?;

    Ok(username)
  }
}

impl<'de> Deserialize<'de> for Username {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_str(UserNameVisitor)
  }
}
