use super::semantic_type::SemanticType;
use nut_webgui_upsmc::Value;

pub struct UnsupportedValueType;

pub trait UnitDisplay: core::fmt::Display {
  type RawValue;

  fn get_semantic_type(&self) -> SemanticType;
  fn set_semantic_type(&mut self, semantic_type: SemanticType);
  fn as_raw_value(&self) -> Self::RawValue;
}

macro_rules! impl_unit {
  ($type:ident, $($rest:tt)+) => {
      impl_unit!(@internal $type, $($rest)+);
  };

  (@internal $type:ident, inner_type = f64 $(, $($rest:tt)+)?) => {
    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    pub struct $type {
      raw_value: f64,
      semantic_type: $crate::http::hypermedia::semantic_type::SemanticType
    }

    impl $crate::http::hypermedia::unit::UnitDisplay for $type {
      type RawValue = f64;

      #[inline]
      fn get_semantic_type(&self) -> $crate::http::hypermedia::semantic_type::SemanticType {
        self.semantic_type
      }

      #[inline]
      fn set_semantic_type(&mut self, semantic_type: $crate::http::hypermedia::semantic_type::SemanticType)  {
        self.semantic_type = semantic_type
      }

      #[inline]
      fn as_raw_value(&self) -> Self::RawValue {
        self.raw_value
      }
    }

    impl AsRef<f64> for $type {
      #[inline]
      fn as_ref(&self) -> &f64 {
        &self.raw_value
      }
    }

    impl From<f64> for $type {
      #[inline]
      fn from(value: f64) -> Self {
        Self {
          raw_value: value,
          semantic_type: $crate::http::hypermedia::semantic_type::SemanticType::Info
        }
      }
    }

    impl TryFrom<Value> for $type {
      type Error = UnsupportedValueType;

      fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.as_lossly_f64() {
          Some(v) => Ok(Self {
            raw_value: v,
            semantic_type: $crate::http::hypermedia::semantic_type::SemanticType::Info,
          }),
          None => Err(UnsupportedValueType),
        }
      }
    }

  impl TryFrom<&Value> for $type {
      type Error = UnsupportedValueType;

      fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_lossly_f64() {
          Some(v) => Ok(Self {
            raw_value: v,
            semantic_type: $crate::http::hypermedia::semantic_type::SemanticType::Info,
          }),
          None => Err(UnsupportedValueType),
        }
      }
    }

    impl_unit!(@internal $type $(, $($rest)+)?);
  };

  (@internal $type:ident, inner_type = i64 $(, $($rest:tt)+)?) => {
    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    pub struct $type {
      raw_value: i64,
      semantic_type: $crate::http::hypermedia::semantic_type::SemanticType
    }

    impl $crate::http::hypermedia::unit::UnitDisplay for $type {
      type RawValue = i64;

      #[inline]
      fn get_semantic_type(&self) -> $crate::http::hypermedia::semantic_type::SemanticType {
        self.semantic_type
      }

      #[inline]
      fn set_semantic_type(&mut self, semantic_type: $crate::http::hypermedia::semantic_type::SemanticType)  {
        self.semantic_type = semantic_type
      }

      #[inline]
      fn as_raw_value(&self) -> Self::RawValue {
        self.raw_value
      }
    }

    impl AsRef<i64> for $type {
      #[inline]
      fn as_ref(&self) -> &i64 {
        &self.raw_value
      }
    }

    impl From<i64> for $type {
      #[inline]
      fn from(value: i64) -> Self {
        Self {
          raw_value: value,
          semantic_type: $crate::http::hypermedia::semantic_type::SemanticType::Info
        }
      }
    }

    impl TryFrom<Value> for $type {
      type Error = UnsupportedValueType;

      fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.as_lossly_i64() {
          Some(v) => Ok(Self {
            raw_value: v,
            semantic_type: $crate::http::hypermedia::semantic_type::SemanticType::Info,
          }),
          None => Err(UnsupportedValueType),
        }
      }
    }

  impl TryFrom<&Value> for $type {
      type Error = UnsupportedValueType;

      fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_lossly_i64() {
          Some(v) => Ok(Self {
            raw_value: v,
            semantic_type: $crate::http::hypermedia::semantic_type::SemanticType::Info,
          }),
          None => Err(UnsupportedValueType),
        }
      }
    }

    impl_unit!(@internal $type $(, $($rest)+)?);
  };


  (@internal $type:ident, format = $format:literal $(, $($rest:tt)+ )?) => {
    impl core::fmt::Display for $type {
      #[inline]
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
          $format,
          self.raw_value,
        ))
      }
    }

    impl askama::FastWritable for $type {
      #[inline]
      fn write_into<W: core::fmt::Write + ?Sized>(
        &self,
        dest: &mut W,
        _: &dyn askama::Values,
      ) -> askama::Result<()> {
        dest.write_fmt(format_args!(
          $format,
          self.raw_value,
        ))?;
        Ok(())
      }
    }

    impl_unit!(@internal $type $(, $($rest)+)?);
  };

  (@internal $type:ident) => {};
}

impl_unit!(ApparentPower, inner_type = f64, format = "{} VA");
impl_unit!(Celcius, inner_type = f64, format = "{} ℃");
impl_unit!(Percentage, inner_type = f64, format = "{} %");
impl_unit!(RealPower, inner_type = f64, format = "{} W");
impl_unit!(RemainingSeconds, inner_type = i64, format = "{} s");
impl_unit!(Voltage, inner_type = f64, format = "{} V");

/// Wrapper type for units with approximate value
#[derive(Debug)]
pub struct Approx<T>
where
  T: UnitDisplay,
{
  inner: T,
}

impl<T> Clone for Approx<T>
where
  T: Clone + UnitDisplay,
{
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

impl<T> Copy for Approx<T> where T: Copy + UnitDisplay {}

impl<T> From<T> for Approx<T>
where
  T: UnitDisplay,
{
  #[inline]
  fn from(value: T) -> Self {
    Self::new(value)
  }
}

impl<T> Approx<T>
where
  T: UnitDisplay,
{
  #[inline]
  pub const fn new(value: T) -> Self {
    Self { inner: value }
  }
}

impl<T> UnitDisplay for Approx<T>
where
  T: UnitDisplay,
{
  type RawValue = T::RawValue;

  fn get_semantic_type(&self) -> SemanticType {
    self.inner.get_semantic_type()
  }

  fn set_semantic_type(&mut self, semantic_type: SemanticType) {
    self.inner.set_semantic_type(semantic_type)
  }

  fn as_raw_value(&self) -> Self::RawValue {
    self.inner.as_raw_value()
  }
}

impl<T> core::fmt::Display for Approx<T>
where
  T: UnitDisplay,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.inner.fmt(f)
  }
}

impl<T> askama::FastWritable for Approx<T>
where
  T: UnitDisplay,
{
  fn write_into<W: core::fmt::Write + ?Sized>(
    &self,
    dest: &mut W,
    _: &dyn askama::Values,
  ) -> askama::Result<()> {
    dest.write_fmt(format_args!("≈ {}", self.inner))?;
    Ok(())
  }
}

#[derive(Debug)]
pub enum OneOf<T1, T2> {
  T1(T1),
  T2(T2),
}

impl<T1, T2> Clone for OneOf<T1, T2>
where
  T1: Clone,
  T2: Clone,
{
  fn clone(&self) -> Self {
    match self {
      Self::T1(arg0) => Self::T1(arg0.clone()),
      Self::T2(arg0) => Self::T2(arg0.clone()),
    }
  }
}

impl<T1, T2> Copy for OneOf<T1, T2>
where
  T1: Copy,
  T2: Copy,
{
}

impl<T1, T2, V> UnitDisplay for OneOf<T1, T2>
where
  T1: UnitDisplay<RawValue = V>,
  T2: UnitDisplay<RawValue = V>,
{
  type RawValue = V;

  #[inline]
  fn get_semantic_type(&self) -> SemanticType {
    match self {
      OneOf::T1(v) => v.get_semantic_type(),
      OneOf::T2(v) => v.get_semantic_type(),
    }
  }

  #[inline]
  fn set_semantic_type(&mut self, semantic_type: SemanticType) {
    match self {
      OneOf::T1(v) => v.set_semantic_type(semantic_type),
      OneOf::T2(v) => v.set_semantic_type(semantic_type),
    }
  }

  #[inline]
  fn as_raw_value(&self) -> Self::RawValue {
    match self {
      OneOf::T1(v) => v.as_raw_value(),
      OneOf::T2(v) => v.as_raw_value(),
    }
  }
}

impl<T1, T2> core::fmt::Display for OneOf<T1, T2>
where
  T1: core::fmt::Display,
  T2: core::fmt::Display,
{
  #[inline]
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      OneOf::T1(v) => v.fmt(f),
      OneOf::T2(v) => v.fmt(f),
    }
  }
}

impl<T1, T2> askama::FastWritable for OneOf<T1, T2>
where
  T1: askama::FastWritable,
  T2: askama::FastWritable,
{
  fn write_into<W: core::fmt::Write + ?Sized>(
    &self,
    dest: &mut W,
    values: &dyn askama::Values,
  ) -> askama::Result<()> {
    match self {
      OneOf::T1(v) => v.write_into(dest, values),
      OneOf::T2(v) => v.write_into(dest, values),
    }
  }
}
