use super::error::InvalidTlsModeError;
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

impl core::str::FromStr for TlsMode {
  type Err = InvalidTlsModeError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_ascii_lowercase().as_str() {
      "disable" => Ok(Self::Disable),
      "skip" => Ok(Self::SkipVerify),
      "strict" => Ok(Self::Strict),
      _ => Err(InvalidTlsModeError),
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

struct TlsModeVisitor;

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
