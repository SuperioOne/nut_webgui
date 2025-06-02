use crate::{errors::UpsNameParseError, internal::escape::escape_nut_str};

/// UPS name
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UpsName {
  name: Box<str>,
}

fn is_ups_name<T>(name: T) -> Result<(), UpsNameParseError>
where
  T: AsRef<str>,
{
  let name = name.as_ref().as_bytes();

  if name.is_empty() {
    Err(UpsNameParseError::Empty)
  } else {
    for byte in name.iter() {
      if byte.is_ascii_whitespace() {
        return Err(UpsNameParseError::InvalidName);
      }
    }

    Ok(())
  }
}

impl UpsName {
  pub fn new<T>(name: T) -> Result<Self, UpsNameParseError>
  where
    T: AsRef<str>,
  {
    _ = is_ups_name(name.as_ref())?;

    Ok(Self {
      name: Box::from(name.as_ref()),
    })
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
  pub fn as_str(&self) -> &str {
    &self.name
  }

  #[inline]
  pub fn into_boxed_str(self) -> Box<str> {
    self.name
  }

  #[inline]
  pub fn as_escaped_str(&self) -> std::borrow::Cow<'_, str> {
    escape_nut_str(&self.name)
  }
}

impl core::str::FromStr for UpsName {
  type Err = UpsNameParseError;

  #[inline]
  fn from_str(value: &str) -> Result<Self, Self::Err> {
    Self::new(value)
  }
}

impl TryFrom<Box<str>> for UpsName {
  type Error = UpsNameParseError;

  #[inline]
  fn try_from(value: Box<str>) -> Result<Self, Self::Error> {
    is_ups_name(value.as_ref())?;

    Ok(Self { name: value })
  }
}

impl TryFrom<String> for UpsName {
  type Error = UpsNameParseError;

  #[inline]
  fn try_from(value: String) -> Result<Self, Self::Error> {
    is_ups_name(&value)?;

    Ok(Self {
      name: value.into_boxed_str(),
    })
  }
}

impl TryFrom<std::borrow::Cow<'_, str>> for UpsName {
  type Error = UpsNameParseError;

  #[inline]
  fn try_from(value: std::borrow::Cow<'_, str>) -> Result<Self, Self::Error> {
    match value {
      std::borrow::Cow::Borrowed(v) => Self::new(v),
      std::borrow::Cow::Owned(v) => Self::try_from(v),
    }
  }
}

impl AsRef<str> for UpsName {
  #[inline]
  fn as_ref(&self) -> &str {
    &self.name
  }
}

impl PartialEq<&str> for UpsName {
  #[inline]
  fn eq(&self, other: &&str) -> bool {
    self.name.as_ref().eq(*other)
  }
}

impl PartialEq<str> for UpsName {
  #[inline]
  fn eq(&self, other: &str) -> bool {
    self.name.as_ref().eq(other)
  }
}

impl PartialEq<String> for UpsName {
  #[inline]
  fn eq(&self, other: &String) -> bool {
    self.name.as_ref().eq(other)
  }
}

impl PartialEq<Box<str>> for UpsName {
  #[inline]
  fn eq(&self, other: &Box<str>) -> bool {
    self.name.as_ref().eq(other.as_ref())
  }
}

impl From<UpsName> for Box<str> {
  #[inline]
  fn from(value: UpsName) -> Self {
    value.name
  }
}

impl std::borrow::Borrow<str> for UpsName {
  fn borrow(&self) -> &str {
    self.name.as_ref()
  }
}

impl std::fmt::Display for UpsName {
  #[inline]
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.name.as_ref())
  }
}

#[cfg(feature = "serde")]
mod serde {
  use super::UpsName;
  use serde::de::Visitor;

  impl serde::Serialize for UpsName {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: serde::Serializer,
    {
      serializer.serialize_str(self.name.as_ref())
    }
  }

  struct UpsNameVisitor;

  impl<'de> serde::Deserialize<'de> for UpsName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: serde::Deserializer<'de>,
    {
      deserializer.deserialize_string(UpsNameVisitor)
    }
  }

  impl<'de> Visitor<'de> for UpsNameVisitor {
    type Value = UpsName;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("expecting an ups name string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      UpsName::new(v).map_err(|err| E::custom(err))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      UpsName::new(v).map_err(|err| E::custom(err))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      UpsName::try_from(v).map_err(|err| E::custom(err))
    }
  }
}
