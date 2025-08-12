use core::{ops::Deref, str::FromStr};
use serde::{Deserialize, de::Visitor};

pub struct PasswordStr(Box<str>);

#[derive(Debug)]
pub struct EmptyPasswordStr;

impl TryFrom<String> for PasswordStr {
  type Error = EmptyPasswordStr;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    if value.trim().is_empty() {
      Err(EmptyPasswordStr)
    } else {
      Ok(PasswordStr(value.into_boxed_str()))
    }
  }
}

impl TryFrom<Box<str>> for PasswordStr {
  type Error = EmptyPasswordStr;

  fn try_from(value: Box<str>) -> Result<Self, Self::Error> {
    if value.trim().is_empty() {
      Err(EmptyPasswordStr)
    } else {
      Ok(PasswordStr(value))
    }
  }
}

impl FromStr for PasswordStr {
  type Err = EmptyPasswordStr;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.trim().is_empty() {
      Err(EmptyPasswordStr)
    } else {
      Ok(PasswordStr(Box::from(s)))
    }
  }
}

impl Drop for PasswordStr {
  fn drop(&mut self) {
    unsafe {
      self.0.as_bytes_mut().fill(0);
    }
  }
}

impl Deref for PasswordStr {
  type Target = str;

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.0.as_ref()
  }
}

struct PasswordStrVisitor;

impl<'de> Visitor<'de> for PasswordStrVisitor {
  type Value = PasswordStr;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("password string")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(PasswordStr(Box::from(v)))
  }

  fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    let pass = PasswordStr::try_from(v).map_err(|err| E::custom(err.to_string()))?;

    Ok(pass)
  }
}

impl<'de> Deserialize<'de> for PasswordStr {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_string(PasswordStrVisitor)
  }
}

impl core::fmt::Display for EmptyPasswordStr {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("password string cannot be empty or whitespace only")
  }
}

impl core::error::Error for EmptyPasswordStr {}
