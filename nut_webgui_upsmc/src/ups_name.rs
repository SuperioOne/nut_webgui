use super::internal::ascii_rules::NutAsciiText;
use crate::errors::UpsNameParseError;
use core::num::NonZeroU16;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Hostname {
  pub name: Box<str>,
  pub port: Option<u16>,
}

impl std::fmt::Display for Hostname {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Hostname {
        name,
        port: Some(port),
      } => write!(f, "{name}:{port}"),
      Hostname { name, .. } => f.write_str(name),
    }
  }
}

impl TryFrom<&str> for Hostname {
  type Error = UpsNameParseError;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    parse_hostname(value)
  }
}

impl Hostname {
  pub fn new<T>(hostname: T) -> Self
  where
    T: AsRef<str>,
  {
    Self {
      port: None,
      name: hostname.as_ref().into(),
    }
  }

  pub fn set_port(mut self, port: NonZeroU16) -> Self {
    self.port = Some(port.get());
    self
  }
}

/// UPS name
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UpsName {
  pub group: Option<Box<str>>,
  pub hostname: Option<Hostname>,
  pub name: Box<str>,
}

fn parse(input: &str) -> Result<UpsName, UpsNameParseError> {
  if input.is_empty() {
    return Err(UpsNameParseError::Empty);
  }

  match input.split_once('@') {
    Some((_, "")) => Err(UpsNameParseError::ExpectedHostname),
    Some(("", _)) => Err(UpsNameParseError::ExpectedUpsName),
    Some((name, hostname)) => {
      let (group_name, ups_name) = parse_ups_name(name)?;
      let hostname = parse_hostname(hostname)?;

      Ok(UpsName {
        name: Box::from(ups_name),
        hostname: Some(hostname),
        group: group_name.map(|v| Box::from(v)),
      })
    }
    None => {
      let (group_name, ups_name) = parse_ups_name(input)?;

      Ok(UpsName {
        name: Box::from(ups_name),
        hostname: None,
        group: group_name.map(|v| Box::from(v)),
      })
    }
  }
}

#[inline]
fn parse_ups_name(input: &str) -> Result<(Option<&str>, &str), UpsNameParseError> {
  match input.split_once(':') {
    Some((_, "")) => Err(UpsNameParseError::ExpectedUpsName),
    Some(("", _)) => Err(UpsNameParseError::ExpectedGroupName),
    Some((group, ups)) => {
      if !is_ups_name(ups) {
        return Err(UpsNameParseError::InvalidUpsName);
      }

      if !is_ups_name(group) {
        return Err(UpsNameParseError::InvalidGroupName);
      }

      Ok((Some(group), ups))
    }
    None => {
      if is_ups_name(input) {
        Ok((None, input))
      } else {
        Err(UpsNameParseError::InvalidUpsName)
      }
    }
  }
}

#[inline]
fn parse_hostname(input: &str) -> Result<Hostname, UpsNameParseError> {
  match input.split_once(':') {
    Some((_, "")) => Err(UpsNameParseError::ExpectedPortNumber),
    Some(("", _)) => Err(UpsNameParseError::ExpectedHostname),
    Some((hostname, port)) => {
      let port: u16 = port
        .parse()
        .map_err(|_| UpsNameParseError::InvalidPortNumber)?;

      Ok(Hostname {
        name: Box::from(hostname),
        port: Some(port),
      })
    }
    None => Ok(Hostname {
      name: Box::from(input),
      port: None,
    }),
  }
}

/// Checks if [`&str`] matches to ups/group/hostname ABNF grammar.
///
/// ```abnf
/// upschar  = DIGIT / ALPHA / 1"_" / 1"-" / 1"."
/// ups      = 1*ALPHA *62upschar
/// group    = ups
/// hostname = ups
/// ```
fn is_ups_name<T>(name: T) -> bool
where
  T: AsRef<str>,
{
  let name = name.as_ref().as_bytes();

  if name.is_empty() {
    false
  } else {
    if let Some(first) = name.get(0) {
      if !first.is_ascii_alphabetic() {
        return false;
      }
    }

    for byte in name.iter() {
      if !byte.is_ascii_nut_ups() {
        return false;
      }
    }

    true
  }
}

impl UpsName {
  pub fn new<T>(name: T) -> Result<Self, UpsNameParseError>
  where
    T: AsRef<str>,
  {
    if !is_ups_name(&name) {
      return Err(UpsNameParseError::InvalidUpsName);
    }

    Ok(Self::new_unchecked(name))
  }

  pub fn new_unchecked<T>(name: T) -> Self
  where
    T: AsRef<str>,
  {
    Self {
      name: Box::from(name.as_ref()),
      group: None,
      hostname: None,
    }
  }

  pub fn set_group<T>(mut self, group: T) -> Result<Self, UpsNameParseError>
  where
    T: AsRef<str>,
  {
    if !is_ups_name(&group) {
      return Err(UpsNameParseError::InvalidGroupName);
    }

    self.group = Some(Box::from(group.as_ref()));

    Ok(self)
  }

  pub fn set_group_unchecked<T>(mut self, group: T) -> Self
  where
    T: AsRef<str>,
  {
    self.group = Some(Box::from(group.as_ref()));
    self
  }

  pub fn set_hostname(mut self, hostname: Hostname) -> Self {
    self.hostname = Some(hostname);
    self
  }
}

impl TryFrom<&str> for UpsName {
  type Error = UpsNameParseError;

  #[inline]
  fn try_from(value: &str) -> Result<Self, Self::Error> {
    parse(value)
  }
}

#[inline]
fn cmp_slice_and_move<'a, 'b>(input: &'a str, target: &'b str) -> Option<&'b str> {
  if input.len() < target.len() {
    let cmp_window = &target[..input.len()];

    if cmp_window == input {
      Some(&target[input.len()..])
    } else {
      None
    }
  } else if input == target {
    Some(target)
  } else {
    None
  }
}

impl PartialEq<str> for Hostname {
  fn eq(&self, other: &str) -> bool {
    let mut rhs_slice: &str = other;

    match cmp_slice_and_move(self.name.as_ref(), rhs_slice) {
      Some(next) => rhs_slice = next,
      None => return false,
    };

    if let Some(port) = self.port.as_ref() {
      match rhs_slice.as_bytes().get(0) {
        Some(b':') => {
          rhs_slice = &rhs_slice[1..];
        }
        _ => return false,
      };

      if &port.to_string() != rhs_slice {
        return false;
      }
    }

    true
  }
}

impl PartialEq<str> for UpsName {
  fn eq(&self, other: &str) -> bool {
    let mut rhs_slice: &str = other;

    if let Some(group) = self.group.as_deref() {
      match cmp_slice_and_move(group, rhs_slice) {
        Some(next) => rhs_slice = next,
        None => return false,
      };

      match rhs_slice.as_bytes().get(0) {
        Some(b':') => {
          rhs_slice = &rhs_slice[1..];
        }
        _ => return false,
      };
    }

    match cmp_slice_and_move(self.name.as_ref(), rhs_slice) {
      Some(next) => rhs_slice = next,
      None => return false,
    };

    if let Some(hostname) = self.hostname.as_ref() {
      match rhs_slice.as_bytes().get(0) {
        Some(b'@') => {
          rhs_slice = &rhs_slice[1..];
        }
        _ => return false,
      };

      return hostname == rhs_slice;
    }

    true
  }
}

impl PartialEq<&str> for UpsName {
  #[inline]
  fn eq(&self, other: &&str) -> bool {
    Self::eq(&self, *other)
  }
}

impl PartialEq<&str> for Hostname {
  #[inline]
  fn eq(&self, other: &&str) -> bool {
    Self::eq(&self, *other)
  }
}

impl std::fmt::Display for UpsName {
  #[inline]
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      UpsName {
        group: Some(group),
        hostname: Some(hostname),
        name,
      } => write!(f, "{group}:{name}@{hostname}"),
      UpsName {
        group: None,
        hostname: None,
        name,
      } => f.write_str(name),
      UpsName {
        group: None,
        hostname: Some(hostname),
        name,
      } => write!(f, "{name}@{hostname}"),
      UpsName {
        group: Some(group),
        hostname: None,
        name,
      } => write!(f, "{group}:{name}"),
    }
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
      serializer.serialize_str(&self.to_string())
    }
  }

  struct UpsNameVisitor;

  impl<'de> serde::Deserialize<'de> for UpsName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: serde::Deserializer<'de>,
    {
      deserializer.deserialize_str(UpsNameVisitor)
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
      UpsName::try_from(v).map_err(|err| E::custom(err.to_string()))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      UpsName::try_from(v).map_err(|err| E::custom(err.to_string()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      UpsName::try_from(v.as_str()).map_err(|err| E::custom(err.to_string()))
    }
  }
}
