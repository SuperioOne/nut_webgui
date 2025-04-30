use crate::internal::ReadOnlyStr;
use std::borrow::Cow;

macro_rules! impl_value_from {
  ($type:ty, $enum:ident) => {
    impl From<$type> for Value {
      fn from(value: $type) -> Self {
        Self::$enum(value)
      }
    }
  };

  ($type:ty, $enum:ident, $conversion:ty) => {
    impl From<$type> for Value {
      fn from(value: $type) -> Self {
        Self::$enum(value as $conversion)
      }
    }
  };
}

/// Basic container type for variable values.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
  Float(f64),
  Int(i64),
  String(ReadOnlyStr),
}

impl Default for Value {
  fn default() -> Self {
    Value::String(ReadOnlyStr::from(""))
  }
}

#[derive(Debug, PartialEq, Eq)]
enum InferredType {
  Int,
  Float,
}

impl Value {
  pub fn infer_from_str(input: &str) -> Self {
    let mut char_iter = input.as_bytes().iter();

    // Checks first char to guess initial value type.
    let mut inferred_type = match char_iter.next() {
      Some(b'-') => InferredType::Int,
      Some(b'0'..=b'9') => InferredType::Int,
      _ => return Self::String(ReadOnlyStr::from(input)),
    };

    for byte in char_iter {
      inferred_type = match *byte {
        b'.' if inferred_type != InferredType::Float => InferredType::Float,
        b'0'..=b'9' => inferred_type,
        _ => return Self::String(ReadOnlyStr::from(input)),
      };
    }

    match inferred_type {
      InferredType::Int => input
        .parse::<i64>()
        .map_or_else(|_| Self::String(ReadOnlyStr::from(input)), |v| Self::Int(v)),

      InferredType::Float => input.parse::<f64>().map_or_else(
        |_| Self::String(ReadOnlyStr::from(input)),
        |v| Self::Float(v),
      ),
    }
  }

  pub fn as_escaped_str(&self) -> Cow<'_, str> {
    match self {
      Value::Float(num) => Cow::Owned(format!("{:.2}", num)),
      Value::Int(num) => Cow::Owned(num.to_string()),
      Value::String(text) => {
        let mut escaped = String::new();
        let mut slice_start: usize = 0;

        for (idx, char_byte) in text.as_bytes().iter().enumerate() {
          if (*char_byte == b'"' || *char_byte == b'\\') && slice_start < idx {
            escaped.push_str(&text[slice_start..idx]);
            slice_start = idx + 1;
          } else {
            continue;
          }
        }

        if escaped.is_empty() {
          Cow::Borrowed(text)
        } else {
          if slice_start < text.len() {
            escaped.push_str(&text[slice_start..]);
          }

          Cow::Owned(escaped)
        }
      }
    }
  }
}

impl std::fmt::Display for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.as_escaped_str())
  }
}

impl_value_from!(f32, Float, f64);
impl_value_from!(i16, Int, i64);
impl_value_from!(i32, Int, i64);
impl_value_from!(i64, Int);
impl_value_from!(i8, Int, i64);

impl From<String> for Value {
  fn from(value: String) -> Self {
    Self::String(ReadOnlyStr::from(value.as_str()))
  }
}

impl From<ReadOnlyStr> for Value {
  fn from(value: ReadOnlyStr) -> Self {
    Self::String(value.clone())
  }
}

impl From<&str> for Value {
  fn from(value: &str) -> Self {
    Self::String(ReadOnlyStr::from(value))
  }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Value {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    match self {
      Value::Float(val) => serializer.serialize_f64(*val),
      Value::Int(val) => serializer.serialize_i64(*val),
      Value::String(val) => serializer.serialize_str(&val),
    }
  }
}
