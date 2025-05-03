use crate::errors::CmdParseError;
use crate::internal::{ReadOnlyStr, ascii_rules::NutAsciiText};

/// INST command name.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct CmdName {
  name: ReadOnlyStr,
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
      name: ReadOnlyStr::from(name.as_ref()),
    }
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

impl TryFrom<&str> for CmdName {
  type Error = CmdParseError;

  #[inline]
  fn try_from(value: &str) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl AsRef<str> for CmdName {
  #[inline]
  fn as_ref(&self) -> &str {
    &self.name
  }
}

impl std::fmt::Display for CmdName {
  #[inline]
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.name)
  }
}

#[cfg(feature = "serde")]
impl serde::Serialize for CmdName {
  #[inline]
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(&self.name)
  }
}
