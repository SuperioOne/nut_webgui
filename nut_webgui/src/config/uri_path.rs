use askama::FastWritable;
use serde::{Deserialize, de::Visitor};

static LOOKUP_ASCII_URI: [bool; 128] = {
  let mut table = [false; 128];
  let mut i: isize = 0;
  let table_ptr = table.as_mut_ptr();

  while i < 128 {
    let cell = unsafe { table_ptr.offset(i) };
    let val = match i as u8 {
      // sub-delims
       b'!'
      | b'$'
      | b'&'
      | b'('
      | b')'
      | b'+'
      | b','
      | b'\''
      | b';'
      | b'='
      // unreserved
      | b'-'
      | b'.'
      | b'_'
      | b'~'
      // Alpha numeric
      | b'a'..=b'z'
      | b'A'..=b'Z'
      | b'0'..=b'9'
      // special terminals
      | b'@'
      | b'/' => true,
      _ => false
          };

    unsafe {
      cell.write(val);
    }

    i += 1;
  }

  table
};

/// See also: [RFC 3986](https://www.rfc-editor.org/rfc/rfc3986#section-3.3)
#[inline]
fn is_valid_path(path: &str) -> bool {
  let mut prev_chr: u8 = b'\0';

  for chr in path.as_bytes() {
    if *chr == b'/' && prev_chr == b'/' {
      return false;
    }

    match LOOKUP_ASCII_URI.get((*chr) as usize) {
      Some(true) => {
        prev_chr = *chr;
        continue;
      }
      _ => return false,
    }
  }

  true
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UriPath {
  inner: String,
}

#[derive(Debug)]
pub struct InvalidPathError;

impl UriPath {
  pub fn new<T>(path: T) -> Result<Self, InvalidPathError>
  where
    T: AsRef<str>,
  {
    let normalized = path
      .as_ref()
      .trim()
      .trim_end_matches(|char: char| char.is_whitespace() || char == '/');

    if normalized.is_empty() {
      Ok(Self::default())
    } else if is_valid_path(normalized) {
      if normalized.starts_with('/') {
        Ok(Self {
          inner: normalized.into(),
        })
      } else {
        Ok(Self {
          inner: format!("/{}", normalized),
        })
      }
    } else {
      Err(InvalidPathError)
    }
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.inner.len()
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }

  #[inline]
  pub fn as_str(&self) -> &str {
    self.inner.as_str()
  }
}

impl Default for UriPath {
  #[inline]
  fn default() -> Self {
    Self {
      inner: String::default(),
    }
  }
}

impl std::fmt::Display for UriPath {
  #[inline]
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.as_str())
  }
}

impl FastWritable for UriPath {
  #[inline]
  fn write_into<W: core::fmt::Write + ?Sized>(
    &self,
    dest: &mut W,
    _values: &dyn askama::Values,
  ) -> askama::Result<()> {
    dest.write_str(self.as_str())?;
    Ok(())
  }
}

impl core::str::FromStr for UriPath {
  type Err = InvalidPathError;

  #[inline]
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::new(s)
  }
}

impl std::fmt::Display for InvalidPathError {
  #[inline]
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("incorrect base path format")
  }
}

impl std::error::Error for InvalidPathError {}

impl AsRef<str> for UriPath {
  #[inline]
  fn as_ref(&self) -> &str {
    &self.inner
  }
}

struct UriPathVisitor;

impl<'de> Visitor<'de> for UriPathVisitor {
  type Value = UriPath;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("expecting uri segment")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    UriPath::new(v).map_err(|err| E::custom(err.to_string()))
  }
}

impl<'de> Deserialize<'de> for UriPath {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_str(UriPathVisitor)
  }
}
