use super::error::ParseTlsModeError;
use serde::{Deserialize, Serialize, de::Visitor};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsMode {
  /// Disable all TLS configurations.
  Disable,

  /// Enables TLS.
  Strict,

  /// Enables TLS, but skips all certificate validations.
  SkipVerify,
}

struct TlsModeVisitor;

impl core::str::FromStr for TlsMode {
  type Err = ParseTlsModeError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.eq_ignore_ascii_case("disable") {
      Ok(Self::Disable)
    } else if s.eq_ignore_ascii_case("skip") {
      Ok(Self::SkipVerify)
    } else if s.eq_ignore_ascii_case("strict") {
      Ok(Self::Strict)
    } else {
      Err(ParseTlsModeError)
    }
  }
}

impl TlsMode {
  pub fn as_str(&self) -> &'static str {
    match self {
      TlsMode::Disable => "disable",
      TlsMode::Strict => "strict",
      TlsMode::SkipVerify => "skip",
    }
  }
}

impl core::fmt::Display for TlsMode {
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str(self.as_str())
  }
}

impl<'de> Visitor<'de> for TlsModeVisitor {
  type Value = TlsMode;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("expecting tls mode option")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    TlsMode::from_str(v).map_err(|err| E::custom(err))
  }
}

impl<'de> Deserialize<'de> for TlsMode {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_str(TlsModeVisitor)
  }
}

impl Serialize for TlsMode {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(self.as_str())
  }
}
