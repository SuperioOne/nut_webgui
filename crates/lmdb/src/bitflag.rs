macro_rules! define_bitflag {
  (
  $(#[$top_level_docs:meta])*
  $access_level:vis $name:ident $inner_type:ty, values = [$(
    $(#[$docs:meta])*
    ($const_name:ident = $flag_value:expr)
  ),+]) => {

    $(#[$top_level_docs])*
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    $access_level struct $name($inner_type);

    impl Default for $name {
      #[inline]
      fn default() -> Self {
          Self::new()
      }
    }

    #[allow(dead_code)]
    impl $name {
      #[inline]
      pub const fn new() -> Self {
          Self(0)
      }

      $(
        $(#[$docs])*
        pub const $const_name: Self = Self($flag_value);
      )+


      /// Returns the internal value of the flag container
      #[inline]
      pub const fn into_inner(&self) -> $inner_type {
        self.0
      }

      /// Checks if all specified bit flags are set in the current instance
      #[inline]
      pub const fn has(&self, rhs: Self) -> bool {
        (self.0 & rhs.0) == rhs.0
      }

      /// Sets specified bit flags
      #[inline]
      pub const fn set(self, value: Self) -> Self {
        Self(self.0 | value.0)
      }

      /// Unsets specified bit flags
      #[inline]
      pub const fn unset(self, value: Self) -> Self {
        Self(self.0 & (!value.0))
      }


      /// Sets specified bit flags in the current instance
      #[inline]
      pub const fn set_assign(&mut self, value: Self) {
        self.0  |= value.0;
      }

      /// Unsets specified bit flags in the current instance
      #[inline]
      pub const fn unset_assign(&mut self, value: Self)  {
        self.0 &= !value.0
      }

      /// Returns the number of set bit flags in the current instance
      #[inline]
      pub const fn len(&self) -> u32 {
        self.0.count_ones()
      }

      /// Checks if no bit flags are set in the current instance
      #[inline]
      pub const fn is_empty(&self) -> bool {
        self.0 == 0
      }
    }

    impl From<$inner_type> for $name {
      #[inline]
      fn from(value: $inner_type) -> Self {
        Self(value)
      }
    }

    impl core::ops::BitOr for $name {
      type Output = $name;
      #[inline]
      fn bitor(self, rhs: Self) -> Self::Output {
        $name(self.0 | rhs.0)
      }
    }

    impl core::ops::BitAnd for $name {
      type Output = $name;
      #[inline]
      fn bitand(self, rhs: $name) -> Self::Output {
        $name(self.0 & rhs.0)
      }
    }

    impl core::ops::BitXor for $name {
      type Output = $name;
      #[inline]
      fn bitxor(self, rhs: Self) -> Self::Output {
        $name(self.0 ^ rhs.0)
      }
    }

    impl core::ops::BitOrAssign for $name {
      #[inline]
      fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
      }
    }

    impl core::ops::BitAndAssign for $name {
      #[inline]
      fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
      }
    }

    impl core::ops::BitXorAssign for $name {
      #[inline]
      fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
      }
    }

    impl core::ops::Not for $name {
      type Output = $name;

      #[inline]
      fn not(self) -> Self::Output {
        Self(!self.0)
      }
    }

    impl core::ops::Shl<$inner_type> for $name {
      type Output = $name;

      #[inline]
      fn shl(self, rhs: $inner_type) -> Self::Output {
        Self(self.0 << rhs)
      }
    }

    impl core::ops::Shr<$inner_type> for $name {
      type Output = $name;

      #[inline]
      fn shr(self, rhs: $inner_type) -> Self::Output {
        Self(self.0 >> rhs)
      }
    }

    impl core::ops::ShrAssign<$inner_type> for $name {
      #[inline]
      fn shr_assign(&mut self, rhs: $inner_type) {
        self.0 >>= rhs;
      }
    }

    impl core::ops::ShlAssign<$inner_type> for $name {
      #[inline]
      fn shl_assign(&mut self, rhs: $inner_type) {
        self.0 <<= rhs;
      }
    }
  };
}

pub(crate) use define_bitflag;
