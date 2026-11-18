use super::{SemanticType, UnitDisplay};

/// Wrapper type for units with approximate value
#[derive(Debug)]
pub struct Approx<T>
where
  T: UnitDisplay,
{
  inner: T,
}

#[derive(Debug)]
pub enum OneOf<T1, T2> {
  T1(T1),
  T2(T2),
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

  fn get(&self) -> Self::RawValue {
    self.inner.get()
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
  fn write_into(
    &self,
    dest: &mut dyn core::fmt::Write,
    _: &dyn askama::Values,
  ) -> askama::Result<()> {
    dest.write_fmt(format_args!("≈ {}", self.inner))?;
    Ok(())
  }
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
  fn get(&self) -> Self::RawValue {
    match self {
      OneOf::T1(v) => v.get(),
      OneOf::T2(v) => v.get(),
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
  fn write_into(
    &self,
    dest: &mut dyn core::fmt::Write,
    values: &dyn askama::Values,
  ) -> askama::Result<()> {
    match self {
      OneOf::T1(v) => v.write_into(dest, values),
      OneOf::T2(v) => v.write_into(dest, values),
    }
  }
}
