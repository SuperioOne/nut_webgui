use super::internal::{ReadOnlyStr, ascii_rules::NutAsciiText};
use crate::errors::UpsNameParseError;
use core::num::NonZeroU16;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Hostname {
  pub name: ReadOnlyStr,
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
  pub group: Option<ReadOnlyStr>,
  pub hostname: Option<Hostname>,
  pub name: ReadOnlyStr,
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
        name: ReadOnlyStr::from(ups_name),
        hostname: Some(hostname),
        group: group_name.map(|v| ReadOnlyStr::from(v)),
      })
    }
    None => {
      let (group_name, ups_name) = parse_ups_name(input)?;

      Ok(UpsName {
        name: ReadOnlyStr::from(ups_name),
        hostname: None,
        group: group_name.map(|v| ReadOnlyStr::from(v)),
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
        name: ReadOnlyStr::from(hostname),
        port: Some(port),
      })
    }
    None => Ok(Hostname {
      name: ReadOnlyStr::from(input),
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
    let name = ReadOnlyStr::from(name.as_ref());

    Self {
      name: ReadOnlyStr::from(name.as_ref()),
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

    self.group = Some(ReadOnlyStr::from(group.as_ref()));

    Ok(self)
  }

  pub fn set_group_unchecked<T>(mut self, group: T) -> Self
  where
    T: AsRef<str>,
  {
    self.group = Some(ReadOnlyStr::from(group.as_ref()));
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
impl serde::Serialize for UpsName {
  #[inline]
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(&self.name)
  }
}
