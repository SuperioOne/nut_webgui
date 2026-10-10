use core::{fmt::Display, marker::PhantomData, str::FromStr};
use serde::de::Visitor;

macro_rules! create_named_enum {
    ($(#[$meta:meta])*
     $vis:vis named_enum $name:ident {
         $(
             $(#[$section_meta:meta])*
             $section:ident = $value:literal
         ),+
     }) => {
        $(#[$meta])*
        $vis enum $name {
            $(
                $(#[$section_meta])*
                $section
            ),+
        }

        impl $name {
            pub const fn as_str(&self) -> &'static str {
                match self {
                    $(
                        $name::$section => $value
                    ),+
                }
            }
        }

        impl $crate::http::hypermedia::util::named_enum::NamedEnum for $name {}

        impl askama::FastWritable for $name {
            fn write_into(&self, dest: &mut dyn core::fmt::Write, _: &dyn askama::Values) -> askama::Result<()> {
                dest.write_str(self.as_str())?;
                Ok(())
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl core::str::FromStr for $name {
            type Err = $crate::http::hypermedia::util::named_enum::NamedEnumParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        $value => Ok(Self::$section),
                    )+
                    _ => Err($crate::http::hypermedia::util::named_enum::NamedEnumParseError)
                }
            }
        }


        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_str($crate::http::hypermedia::util::named_enum::NamedEnumVisitor::<$name>::new())
            }
        }
    };
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NamedEnumParseError;

pub trait NamedEnum: FromStr<Err = NamedEnumParseError> + Display {}

#[derive(Clone, Copy, Default)]
pub struct NamedEnumVisitor<T> {
  _phantom: PhantomData<T>,
}

impl<T> NamedEnumVisitor<T> {
  pub const fn new() -> Self {
    Self {
      _phantom: PhantomData,
    }
  }
}

impl<'de, T> Visitor<'de> for NamedEnumVisitor<T>
where
  T: NamedEnum,
{
  type Value = T;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("expecting named enum")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    T::from_str(v).map_err(|_| E::custom("invalid named enum"))
  }

  #[inline]
  fn visit_borrowed_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    self.visit_str(v)
  }

  #[inline]
  fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    self.visit_str(v.as_str())
  }
}

pub(crate) use create_named_enum;
