use crate::errors::VarTypeParseError;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum VarType {
  ReadWrite,
  Enum,
  Range,
  String { max_len: usize },
  Number,
}

impl core::str::FromStr for VarType {
  type Err = VarTypeParseError;

  #[inline]
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.is_empty() {
      return Err(VarTypeParseError::Empty);
    }

    match s {
      "RW" => Ok(Self::ReadWrite),
      "ENUM" => Ok(Self::Enum),
      "RANGE" => Ok(Self::Range),
      "NUMBER" => Ok(Self::Number),
      text => {
        const STR_START: &'static str = "STRING:";

        if text.len() > STR_START.len() && text.starts_with(STR_START) {
          match (&text[STR_START.len()..]).parse() {
            Ok(v) => Ok(Self::String { max_len: v }),
            Err(_) => Err(VarTypeParseError::InvalidType),
          }
        } else {
          Err(VarTypeParseError::InvalidType)
        }
      }
    }
  }
}

impl std::fmt::Display for VarType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      VarType::ReadWrite => f.write_str("RW"),
      VarType::Enum => f.write_str("RW"),
      VarType::Range => f.write_str("RANGE"),
      VarType::String { max_len } => f.write_fmt(format_args!("STRING:{max_len}")),
      VarType::Number => f.write_str("NUMBER"),
    }
  }
}
