use super::{SemanticType, UnitDisplay};
use upsmc::Value;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UnsupportedValueType;

#[inline]
fn trim_f64_str(value: &str) -> &str {
  value
    .trim_end_matches(|c| c == '0')
    .trim_end_matches(|c| c == '.')
}

macro_rules! impl_unit {
  ($vis:vis $name:ident($type:tt, $format:literal)) => {

    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    $vis struct $name {
      raw_value: $type,
      semantic_type: SemanticType
    }

    impl UnitDisplay for $name {
      type RawValue = $type;

      #[inline]
      fn get_semantic_type(&self) -> SemanticType {
        self.semantic_type
      }

      #[inline]
      fn set_semantic_type(&mut self, semantic_type: SemanticType)  {
        self.semantic_type = semantic_type
      }

      #[inline]
      fn get(&self) -> Self::RawValue {
        self.raw_value
      }
    }

    impl_unit!(@internal $name, $type, $format);
  };

  (@internal $name:ident, f64, $format:literal) => {
    impl AsRef<f64> for $name {
      #[inline]
      fn as_ref(&self) -> &f64 {
        &self.raw_value
      }
    }

    impl From<f64> for $name {
      #[inline]
      fn from(value: f64) -> Self {
        Self {
          raw_value: value,
          semantic_type: SemanticType::Info
        }
      }
    }

    impl TryFrom<Value> for $name {
      type Error = UnsupportedValueType;

      #[inline]
      fn try_from(value: Value) -> Result<Self, Self::Error> {
        $name::try_from(&value)
      }
    }

    impl TryFrom<&Value> for $name {
      type Error = UnsupportedValueType;

      fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_lossy_f64() {
          Some(v) => Ok(Self {
            raw_value: v,
            semantic_type: SemanticType::Info,
          }),
          None => Err(UnsupportedValueType),
        }
      }
    }

    impl core::fmt::Display for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatted = format!("{:.2}", self.raw_value);
        f.write_fmt(format_args!($format, trim_f64_str(&formatted)))
      }
    }

    impl askama::FastWritable for $name {
      fn write_into(
        &self,
        dest: &mut dyn core::fmt::Write,
        _: &dyn askama::Values,
      ) -> askama::Result<()> {
        let formatted = format!("{:.2}", self.raw_value);
        dest.write_fmt(format_args!($format, trim_f64_str(&formatted)))?;
        Ok(())
      }
    }

  };

  (@internal $name:ident, i64, $format:literal) => {
    impl AsRef<i64> for $name {
      #[inline]
      fn as_ref(&self) -> &i64 {
        &self.raw_value
      }
    }

    impl From<i64> for $name {
      #[inline]
      fn from(value: i64) -> Self {
        Self {
          raw_value: value,
          semantic_type: SemanticType::Info
        }
      }
    }

    impl TryFrom<Value> for $name {
      type Error = UnsupportedValueType;

      #[inline]
      fn try_from(value: Value) -> Result<Self, Self::Error> {
        $name::try_from(&value)
      }
    }

  impl TryFrom<&Value> for $name {
      type Error = UnsupportedValueType;

      fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_lossy_i64() {
          Some(v) => Ok(Self {
            raw_value: v,
            semantic_type: SemanticType::Info,
          }),
          None => Err(UnsupportedValueType),
        }
      }
    }

    impl core::fmt::Display for $name {
      #[inline]
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
          $format,
          self.raw_value,
        ))
      }
    }

    impl askama::FastWritable for $name {
      #[inline]
      fn write_into(
        &self,
        dest: &mut dyn core::fmt::Write,
        _: &dyn askama::Values,
      ) -> askama::Result<()> {
        dest.write_fmt(format_args!(
          $format,
          self.raw_value,
        ))?;
        Ok(())
      }
    }
  };
}

impl_unit!(pub ApparentPower(f64, "{} VA"));
impl_unit!(pub Celcius(f64 ,"{} °C"));
impl_unit!(pub Percentage(f64, "{} %"));
impl_unit!(pub RealPower(f64, "{} W"));
impl_unit!(pub Voltage(f64, "{} V"));
impl_unit!(pub Second(i64, "{} s"));
