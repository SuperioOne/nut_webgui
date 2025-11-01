use core::{num::ParseIntError, str::FromStr};
use serde::{Deserialize, de, de::Visitor};
use std::fmt::Write;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Permissions(u8);

impl Default for Permissions {
  #[inline]
  fn default() -> Self {
    Self(0)
  }
}

impl Permissions {
  /// User can set forced shutdown flag on UPS
  pub const FSD: Permissions = Permissions(1);

  /// User can call instant command on UPS
  pub const INSTCMD: Permissions = Permissions(2);

  /// User can modify RW variables on UPS
  pub const SETVAR: Permissions = Permissions(4);

  #[inline]
  pub const fn all() -> Self {
    Self::FSD.set(Self::INSTCMD).set(Self::SETVAR)
  }

  #[inline]
  pub const fn has(&self, rhs: Permissions) -> bool {
    (self.0 & rhs.0) == rhs.0
  }

  #[inline]
  pub const fn set(self, value: Permissions) -> Self {
    Self(self.0 | value.0)
  }

  #[inline]
  pub const fn unset(self, value: Permissions) -> Self {
    Self(self.0 & (!value.0))
  }

  #[inline]
  pub const fn len(&self) -> u32 {
    self.0.count_ones()
  }

  #[inline]
  pub const fn is_empty(&self) -> bool {
    self.0 == 0
  }

  #[inline]
  pub fn iter(&self) -> Iter {
    Iter { state: *self }
  }

  #[inline]
  pub const fn as_u8(&self) -> u8 {
    self.0
  }

  pub const fn as_str(&self) -> &'static str {
    match *self {
      Self::FSD => "fsd",
      Self::INSTCMD => "instcmd",
      Self::SETVAR => "setvar",
      _ => "unknown permission",
    }
  }
}

impl From<u8> for Permissions {
  #[inline]
  fn from(value: u8) -> Self {
    Self(value)
  }
}

impl core::ops::BitOr for Permissions {
  type Output = Permissions;

  #[inline]
  fn bitor(self, rhs: Self) -> Self::Output {
    Permissions(self.0 | rhs.0)
  }
}

impl core::ops::BitAnd for Permissions {
  type Output = Permissions;

  #[inline]
  fn bitand(self, rhs: Self) -> Self::Output {
    Permissions(self.0 & rhs.0)
  }
}

impl core::ops::BitXor for Permissions {
  type Output = Permissions;

  #[inline]
  fn bitxor(self, rhs: Self) -> Self::Output {
    Permissions(self.0 ^ rhs.0)
  }
}

impl core::ops::BitOrAssign for Permissions {
  #[inline]
  fn bitor_assign(&mut self, rhs: Self) {
    self.0 |= rhs.0;
  }
}

impl core::ops::BitAndAssign for Permissions {
  #[inline]
  fn bitand_assign(&mut self, rhs: Self) {
    self.0 &= rhs.0;
  }
}

impl core::ops::BitXorAssign for Permissions {
  #[inline]
  fn bitxor_assign(&mut self, rhs: Self) {
    self.0 ^= rhs.0;
  }
}

pub struct Iter {
  state: Permissions,
}

impl Iterator for Iter {
  type Item = Permissions;

  fn next(&mut self) -> Option<Self::Item> {
    if self.state.0 == 0 {
      None
    } else {
      let mask: u8 = 1 << self.state.0.trailing_zeros();
      let yield_val = Permissions(self.state.0 & mask);
      self.state = self.state.unset(yield_val);

      Some(yield_val)
    }
  }
}

impl core::str::FromStr for Permissions {
  type Err = ParseIntError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Permissions::from(u8::from_str(s)?))
  }
}

struct PermissionVisitor;

impl<'de> Visitor<'de> for PermissionVisitor {
  type Value = Permissions;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str(
      "expecting sequence of permissions: fsd, instcmd, setvar, or permission flag as number",
    )
  }

  fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::SeqAccess<'de>,
  {
    let mut permission = Permissions::default();

    while let Some(element) = seq.next_element::<String>()? {
      if element.eq_ignore_ascii_case("fsd") {
        permission = permission.set(Permissions::FSD);
      } else if element.eq_ignore_ascii_case("instcmd") {
        permission = permission.set(Permissions::INSTCMD);
      } else if element.eq_ignore_ascii_case("setvar") {
        permission = permission.set(Permissions::SETVAR);
      } else {
        return Err(de::Error::custom("invalid permission type"));
      }
    }

    Ok(permission)
  }

  fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Permissions::from(v))
  }

  fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Permissions::from(v as u8))
  }

  fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Permissions::from(v as u8))
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Permissions::from_str(v).map_err(|err| E::custom(err.to_string()))
  }
}

impl<'de> Deserialize<'de> for Permissions {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_any(PermissionVisitor)
  }
}

impl askama::FastWritable for Permissions {
  fn write_into<W: core::fmt::Write + ?Sized>(
    &self,
    dest: &mut W,
    _: &dyn askama::Values,
  ) -> askama::Result<()> {
    if self.len() > 1 {
      let mut iter = self.iter().peekable();

      while let Some(value) = iter.next() {
        dest.write_str(value.as_str())?;

        if iter.peek().is_some() {
          dest.write_char(' ')?;
        }
      }

      Ok(())
    } else {
      dest.write_str((*self).as_str())?;
      Ok(())
    }
  }
}

impl std::fmt::Display for Permissions {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.len() > 1 {
      let mut iter = self.iter().peekable();

      while let Some(value) = iter.next() {
        f.write_str(value.as_str())?;

        if iter.peek().is_some() {
          f.write_char(' ')?;
        }
      }

      Ok(())
    } else {
      f.write_str((*self).as_str())
    }
  }
}
