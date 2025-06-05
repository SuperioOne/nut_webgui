#[derive(Debug, Clone, Copy)]
pub enum SemanticType {
  Info,
  Error,
  Warning,
  Success,
}

impl SemanticType {
  #[inline]
  pub fn as_badge(self) -> &'static str {
    BadgeStyle::from_type(self)
  }

  #[inline]
  pub fn as_text(self) -> &'static str {
    TextStyle::from_type(self)
  }

  #[inline]
  pub fn as_fill(self) -> &'static str {
    FillStyle::from_type(self)
  }

  #[inline]
  pub fn as_progress(self) -> &'static str {
    ProgressStyle::from_type(self)
  }
}

pub trait SemanticClass {
  fn error() -> &'static str;
  fn info() -> &'static str;
  fn success() -> &'static str;
  fn warning() -> &'static str;
  fn from_type(value: SemanticType) -> &'static str;
}

impl SemanticType {
  #[inline]
  pub fn from_range<T>(value: T, from: T, to: T) -> Self
  where
    T: PartialOrd + PartialEq,
  {
    let level = (value >= from) as u8 + (value >= to) as u8;

    match level {
      0 => SemanticType::Success,
      1 => SemanticType::Warning,
      2 => SemanticType::Error,
      _ => unreachable!(),
    }
  }

  #[inline]
  pub fn from_range_inverted<T>(value: T, from: T, to: T) -> Self
  where
    T: PartialOrd + PartialEq,
  {
    let level = (value >= from) as u8 + (value >= to) as u8;

    match level {
      0 => SemanticType::Error,
      1 => SemanticType::Warning,
      2 => SemanticType::Success,
      _ => unreachable!(),
    }
  }
}

macro_rules! impl_semantic_class {
  ($struct_name:ident, { info = $info:literal, success = $success:literal, warning = $warning:literal, error = $error:literal }) => {
    impl SemanticClass for $struct_name {
      #[inline]
      fn info() -> &'static str {
        $info
      }

      #[inline]
      fn warning() -> &'static str {
        $warning
      }

      #[inline]
      fn success() -> &'static str {
        $success
      }

      #[inline]
      fn error() -> &'static str {
        $error
      }

      #[inline]
      fn from_type(value: SemanticType) -> &'static str {
        match value {
          SemanticType::Info => $info,
          SemanticType::Error => $error,
          SemanticType::Warning => $warning,
          SemanticType::Success => $success,
        }
      }
    }
  };
}

pub struct TextStyle;
pub struct BadgeStyle;
pub struct FillStyle;
pub struct ProgressStyle;

impl_semantic_class!(
  TextStyle,
  {
    info = "text-info",
    success = "text-success",
    warning = "text-warning",
    error = "text-error"
  }
);

impl_semantic_class!(
  BadgeStyle,
  {
    info = "badge-info",
    success = "badge-success",
    warning = "badge-warning",
    error = "badge-error"
  }

);

impl_semantic_class!(
  FillStyle,
  {
    info = "fill-info",
    success = "fill-success",
    warning = "fill-warning",
    error = "fill-error"
  }
);

impl_semantic_class!(
  ProgressStyle,
  {
    info = "progress-info",
    success = "progress-success",
    warning = "progress-warning",
    error = "progress-error"
  }
);
