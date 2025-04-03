use crate::internal::ReadOnlyStr;

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
  Boolean(bool),
  Float(f64),
  Int(i64),
  String(ReadOnlyStr),
}

impl std::fmt::Display for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Value::Boolean(val) => val.fmt(f),
      Value::Float(val) => val.fmt(f),
      Value::Int(val) => val.fmt(f),
      Value::String(val) => val.fmt(f),
    }
  }
}

impl_value_from!(bool, Boolean);
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
      Value::Boolean(val) => serializer.serialize_bool(*val),
      Value::Float(val) => serializer.serialize_f64(*val),
      Value::Int(val) => serializer.serialize_i64(*val),
      Value::String(val) => serializer.serialize_str(&val),
    }
  }
}
