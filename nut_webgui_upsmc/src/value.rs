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
  String(Box<str>),
}

#[derive(Debug, PartialEq, Eq)]
enum InferredType {
  Int,
  Float,
  String,
}

impl Value {
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

fn infer_type(input: &str) -> InferredType {
  let mut char_iter = input.as_bytes().iter();

  // Checks first char to guess initial value type.
  let mut inferred_type = match char_iter.next() {
    Some(b'-') => InferredType::Int,
    Some(b'0') => {
      // Second char
      match char_iter.next() {
        Some(b'.') => InferredType::Float,
        Some(_) => return InferredType::String,
        None => InferredType::Int,
      }
    }
    Some(b'0'..=b'9') => InferredType::Int,
    _ => return InferredType::String,
  };

  for byte in char_iter {
    inferred_type = match *byte {
      b'.' if inferred_type == InferredType::Int => InferredType::Float,
      b'0'..=b'9' => inferred_type,
      _ => return InferredType::String,
    };
  }

  inferred_type
}

pub trait InferValueFrom<T> {
  fn infer_from(value: T) -> Value;
}

impl InferValueFrom<Box<str>> for Value {
  fn infer_from(value: Box<str>) -> Value {
    match infer_type(&value) {
      InferredType::Int => value
        .parse::<i64>()
        .map_or_else(|_| Self::String(value), |v| Self::Int(v)),

      InferredType::Float => value
        .parse::<f64>()
        .map_or_else(|_| Self::String(value), |v| Self::Float(v)),

      _ => Self::String(value),
    }
  }
}

impl InferValueFrom<String> for Value {
  fn infer_from(value: String) -> Value {
    match infer_type(&value) {
      InferredType::Int => value
        .parse::<i64>()
        .map_or_else(|_| Self::String(value.into_boxed_str()), |v| Self::Int(v)),

      InferredType::Float => value
        .parse::<f64>()
        .map_or_else(|_| Self::String(value.into_boxed_str()), |v| Self::Float(v)),

      _ => Self::String(value.into_boxed_str()),
    }
  }
}

impl InferValueFrom<&str> for Value {
  fn infer_from(value: &str) -> Value {
    match infer_type(&value) {
      InferredType::Int => value
        .parse::<i64>()
        .map_or_else(|_| Self::String(Box::from(value)), |v| Self::Int(v)),

      InferredType::Float => value
        .parse::<f64>()
        .map_or_else(|_| Self::String(Box::from(value)), |v| Self::Float(v)),

      _ => Self::String(Box::from(value)),
    }
  }
}

impl InferValueFrom<Cow<'_, str>> for Value {
  #[inline]
  fn infer_from(value: Cow<'_, str>) -> Value {
    match value {
      Cow::Borrowed(v) => Self::infer_from(v),
      Cow::Owned(v) => Self::infer_from(v),
    }
  }
}

impl std::fmt::Display for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.as_escaped_str())
  }
}

impl_value_from!(f64, Float);
impl_value_from!(f32, Float, f64);

impl_value_from!(i16, Int, i64);
impl_value_from!(i32, Int, i64);
impl_value_from!(i64, Int);
impl_value_from!(i8, Int, i64);

impl_value_from!(u16, Int, i64);
impl_value_from!(u32, Int, i64);
impl_value_from!(u64, Int, i64);
impl_value_from!(u8, Int, i64);

impl From<String> for Value {
  #[inline]
  fn from(value: String) -> Self {
    Self::String(value.into_boxed_str())
  }
}

impl From<Box<str>> for Value {
  #[inline]
  fn from(value: Box<str>) -> Self {
    Self::String(value)
  }
}

impl From<Cow<'_, str>> for Value {
  #[inline]
  fn from(value: Cow<'_, str>) -> Self {
    match value {
      Cow::Borrowed(v) => Self::from(v),
      Cow::Owned(v) => Self::from(v),
    }
  }
}

impl From<&str> for Value {
  #[inline]
  fn from(value: &str) -> Self {
    Self::String(Box::from(value))
  }
}

impl PartialEq<i64> for Value {
  #[inline]
  fn eq(&self, other: &i64) -> bool {
    match self {
      Value::Float(_) => false,
      Value::Int(val) => val.eq(other),
      Value::String(_) => false,
    }
  }
}

impl PartialEq<f64> for Value {
  #[inline]
  fn eq(&self, other: &f64) -> bool {
    match self {
      Value::Float(val) => val.eq(other),
      Value::Int(_) => false,
      Value::String(_) => false,
    }
  }
}

impl PartialEq<str> for Value {
  #[inline]
  fn eq(&self, other: &str) -> bool {
    match self {
      Value::Float(_) => false,
      Value::Int(_) => false,
      Value::String(inner) => inner.as_ref().eq(other),
    }
  }
}

impl PartialEq<Box<str>> for Value {
  #[inline]
  fn eq(&self, other: &Box<str>) -> bool {
    match self {
      Value::Float(_) => false,
      Value::Int(_) => false,
      Value::String(inner) => inner.as_ref().eq(other.as_ref()),
    }
  }
}

impl PartialEq<String> for Value {
  #[inline]
  fn eq(&self, other: &String) -> bool {
    match self {
      Value::Float(_) => false,
      Value::Int(_) => false,
      Value::String(inner) => inner.as_ref().eq(other),
    }
  }
}

impl PartialEq<&str> for Value {
  #[inline]
  fn eq(&self, other: &&str) -> bool {
    self == *other
  }
}

impl PartialEq<i64> for &Value {
  #[inline]
  fn eq(&self, other: &i64) -> bool {
    **self == *other
  }
}

impl PartialEq<f64> for &Value {
  #[inline]
  fn eq(&self, other: &f64) -> bool {
    **self == *other
  }
}

impl PartialEq<str> for &Value {
  #[inline]
  fn eq(&self, other: &str) -> bool {
    **self == *other
  }
}

impl PartialEq<Box<str>> for &Value {
  #[inline]
  fn eq(&self, other: &Box<str>) -> bool {
    **self == *other
  }
}

impl PartialEq<String> for &Value {
  #[inline]
  fn eq(&self, other: &String) -> bool {
    **self == *other
  }
}

#[cfg(feature = "serde")]
mod serde {
  use super::Value;
  use serde::de::Visitor;

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

  struct ValueVisitor;

  impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: serde::Deserializer<'de>,
    {
      deserializer.deserialize_any(ValueVisitor)
    }
  }

  macro_rules! impl_visit_for {
    ($name:ident, $type:ty) => {
      fn $name<E>(self, v: $type) -> Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        Ok(Value::from(v))
      }
    };
  }

  impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("expecting number, float, or name")
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      Ok(Value::String(Box::from(v)))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      Ok(Value::String(v.into_boxed_str()))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      Ok(Value::String(Box::from(v)))
    }

    impl_visit_for!(visit_f32, f32);
    impl_visit_for!(visit_f64, f64);

    impl_visit_for!(visit_i16, i16);
    impl_visit_for!(visit_i32, i32);
    impl_visit_for!(visit_i64, i64);
    impl_visit_for!(visit_i8, i8);

    impl_visit_for!(visit_u16, u16);
    impl_visit_for!(visit_u32, u32);
    impl_visit_for!(visit_u64, u64);
    impl_visit_for!(visit_u8, u8);
  }
}
