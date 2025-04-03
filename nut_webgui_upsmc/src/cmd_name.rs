use crate::errors::ParseErrors;
use crate::internal::{ReadOnlyStr, ascii_rules::NutAsciiText};

/// INST command name.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct CmdName {
  name: ReadOnlyStr,
}

/// Checks if [`&str`] matches to cmdname ABNF grammar.
///
/// ```abnf
/// varname = 1*LOWERCASE_ASCII *62( DOT 1*LOWERCASE_ASCII )
/// ```
fn is_cmd_name<T>(name: T) -> Result<(), ParseErrors>
where
  T: AsRef<str>,
{
  let name = name.as_ref().as_bytes();

  if name.is_empty() {
    Err(ParseErrors::Empty)
  } else if name.len() > 63 {
    Err(ParseErrors::OutOfBounds)
  } else if let Some(b'.') = name.get(0) {
    Err(ParseErrors::InvalidChar { position: 0 })
  } else {
    for (idx, byte) in name.iter().enumerate() {
      if !byte.is_ascii_nut_cmd() {
        return Err(ParseErrors::InvalidChar { position: idx });
      }
    }

    Ok(())
  }
}

impl CmdName {
  pub fn new<T>(name: T) -> Result<Self, ParseErrors>
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
