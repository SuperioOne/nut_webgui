use crate::errors::CmdParseError;
use crate::internal::ascii_rules::NutAsciiText;
use core::borrow::Borrow;

/// INST command name.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct CmdName {
  name: Box<str>,
}

/// Checks if [`&str`] matches to cmdname ABNF grammar.
///
/// ```abnf
/// cmdname = 1*LOWERCASE_ASCII *62( DOT 1*LOWERCASE_ASCII )
/// ```
fn is_cmd_name<T>(name: T) -> Result<(), CmdParseError>
where
  T: AsRef<str>,
{
  let name = name.as_ref().as_bytes();

  if name.is_empty() {
    return Err(CmdParseError::Empty);
  }

  if let Some(first) = name.get(0) {
    if !first.is_ascii_alphabetic() {
      return Err(CmdParseError::InvalidName);
    }
  }

  for byte in name.iter() {
    if !byte.is_ascii_nut_cmd() {
      return Err(CmdParseError::InvalidName);
    }
  }

  Ok(())
}

impl CmdName {
  pub fn new<T>(name: T) -> Result<Self, CmdParseError>
  where
    T: AsRef<str>,
  {
    is_cmd_name(&name)?;

    Ok(Self::new_unchecked(name))
  }

  pub fn new_unchecked<T>(name: T) -> Self
  where
    T: AsRef<str>,
  {
    Self {
      name: Box::from(name.as_ref()),
    }
  }

  #[inline]
  pub fn into_boxed_str(self) -> Box<str> {
    self.name
  }

  #[inline]
  pub const fn as_str(&self) -> &str {
    &self.name
  }

  #[inline]
  pub fn is_valid_name(name: &str) -> bool {
    is_cmd_name(name).is_ok()
  }
}

impl core::str::FromStr for CmdName {
  type Err = CmdParseError;

  #[inline]
  fn from_str(value: &str) -> Result<Self, Self::Err> {
    Self::new(value)
  }
}

impl TryFrom<Box<str>> for CmdName {
  type Error = CmdParseError;

  #[inline]
  fn try_from(value: Box<str>) -> Result<Self, Self::Error> {
    is_cmd_name(&value)?;

    Ok(Self { name: value })
  }
}

impl TryFrom<std::borrow::Cow<'_, str>> for CmdName {
  type Error = CmdParseError;

  #[inline]
  fn try_from(value: std::borrow::Cow<'_, str>) -> Result<Self, Self::Error> {
    match value {
      std::borrow::Cow::Borrowed(v) => Self::new(v),
      std::borrow::Cow::Owned(v) => Self::try_from(v),
    }
  }
}

impl TryFrom<String> for CmdName {
  type Error = CmdParseError;

  #[inline]
  fn try_from(value: String) -> Result<Self, Self::Error> {
    is_cmd_name(&value)?;

    Ok(Self {
      name: value.into_boxed_str(),
    })
  }
}

impl AsRef<str> for CmdName {
  #[inline]
  fn as_ref(&self) -> &str {
    &self.name
  }
}

impl PartialEq<str> for CmdName {
  #[inline]
  fn eq(&self, other: &str) -> bool {
    self.name.as_ref().eq(other)
  }
}

impl PartialEq<String> for CmdName {
  #[inline]
  fn eq(&self, other: &String) -> bool {
    self.name.as_ref().eq(other)
  }
}

impl PartialEq<Box<str>> for CmdName {
  #[inline]
  fn eq(&self, other: &Box<str>) -> bool {
    self.name.as_ref().eq(other.as_ref())
  }
}

impl From<CmdName> for Box<str> {
  #[inline]
  fn from(value: CmdName) -> Self {
    value.into_boxed_str()
  }
}

impl Borrow<str> for CmdName {
  fn borrow(&self) -> &str {
    self.as_str()
  }
}

impl std::fmt::Display for CmdName {
  #[inline]
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.name)
  }
}

#[cfg(feature = "serde")]
mod serde {
  use super::CmdName;
  use serde::de::Visitor;

  impl serde::Serialize for CmdName {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: serde::Serializer,
    {
      serializer.serialize_str(&self.name)
    }
  }

  struct CmdNameVisitor;

  impl<'de> serde::Deserialize<'de> for CmdName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: serde::Deserializer<'de>,
    {
      deserializer.deserialize_string(CmdNameVisitor)
    }
  }

  impl<'de> Visitor<'de> for CmdNameVisitor {
    type Value = CmdName;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("expecting an cmd name string")
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      CmdName::new(v).map_err(|err| E::custom(err.to_string()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      CmdName::try_from(v).map_err(|err| E::custom(err.to_string()))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      CmdName::new(v).map_err(|err| E::custom(err.to_string()))
    }
  }
}
