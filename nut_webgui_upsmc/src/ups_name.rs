use super::internal::{ReadOnlyStr, ascii_rules::NutAsciiText};
use crate::errors::ParseErrors;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Hostname {
  name: ReadOnlyStr,
  port: Option<u16>,
}

/// UPS name
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UpsName {
  group: Option<ReadOnlyStr>,
  hostname: Option<Hostname>,
  name: ReadOnlyStr,
}

/// Checks if [`&str`] matches to ups/group/hostname ABNF grammar.
///
/// ```abnf
/// upschar  = DIGIT / ALPHA / 1"_" / 1"-" / 1"."
/// ups      = 1*ALPHA *62upschar
/// group    = ups
/// hostname = ups
/// ```
fn is_ups_name<T>(name: T) -> Result<(), ParseErrors>
where
  T: AsRef<str>,
{
  let name = name.as_ref().as_bytes();

  if name.is_empty() {
    Err(ParseErrors::Empty)
  } else if name.len() > 63 {
    Err(ParseErrors::OutOfBounds)
  } else {
    _ = match name.get(0) {
      Some(first) if !first.is_ascii_alphabetic() => Ok(()),
      _ => Err(ParseErrors::InvalidChar { position: 0 }),
    }?;

    for (idx, byte) in name.iter().enumerate() {
      if !byte.is_ascii_nut_ups() {
        return Err(ParseErrors::InvalidChar { position: idx });
      }
    }

    Ok(())
  }
}

impl UpsName {
  pub fn new<T>(name: T, group: Option<T>, hostname: Option<Hostname>) -> Result<Self, ParseErrors>
  where
    T: AsRef<str>,
  {
    is_ups_name(&name)?;

    Ok(Self::new_unchecked(name, group, hostname))
  }

  pub fn new_unchecked<T, G>(name: T, group: Option<G>, hostname: Option<Hostname>) -> Self
  where
    T: AsRef<str>,
    G: AsRef<str>,
  {
    let name = ReadOnlyStr::from(name.as_ref());
    let group = group.map(|v| ReadOnlyStr::from(v.as_ref()));

    Self {
      name,
      group,
      hostname,
    }
  }

  #[inline]
  pub fn is_valid_name(name: &str) -> bool {
    is_ups_name(name).is_ok()
  }
}

impl std::fmt::Display for UpsName {
  #[inline]
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.name)
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
