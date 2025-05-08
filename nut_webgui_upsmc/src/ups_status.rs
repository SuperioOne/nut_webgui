use crate::errors::UnsupportedStatusError;

// Implements repetitive traits and const values
macro_rules! impl_status {
  ($(($name:ident, $value:literal);)+) => {
    impl $crate::ups_status::UpsStatus {
      impl_status!(@const_val 0, $(($name);)+);

      pub fn from_str_unchecked<T>(value:T) -> Self
        where
          T: AsRef<str>
      {
        let mut result = $crate::ups_status::UpsStatus::default();

        for status in value.as_ref().split_whitespace() {
          match status {
            $(
              $value => { result |= $crate::ups_status::UpsStatus::$name },
            )+
            _ => {}
          };
        }

        result
      }
    }

    impl TryFrom<&str> for UpsStatus {
      type Error = $crate::errors::UnsupportedStatusError;

      fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut result = Self::default();

        for status in value.split_whitespace() {
          match status {
            $(
              $value => { result |= Self::$name },
            )+
            _ => return Err($crate::errors::UnsupportedStatusError { status: Box::from(status)})
          };
        }

        Ok(result)
      }
    }

    fn get_state_str(value: $crate::ups_status::UpsStatus) -> &'static str {
      match value {
        $crate::ups_status::UpsStatus(0) => "",
        $(
          $crate::ups_status::UpsStatus::$name => $value,
        )+
        _ => "UnknownStatus",
      }
    }
  };

  (@const_val $order:expr, ($name:ident); $($rest:tt)*) => {
      pub const $name: $crate::ups_status::UpsStatus = $crate::ups_status::UpsStatus(1u32 << ($order));
      impl_status!(@const_val $order + 1, $($rest)*);
  };

  (@const_val $order:expr,) => { };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UpsStatus(u32);

impl_status!(
(ALARM,           "ALARM");
(BOOST,           "BOOST");
(BYPASS,          "BYPASS");
(CALIBRATING,     "CAL");
(CHARGING,        "CHRG");
(COMM,            "COMM");
(DISCHARGE,       "DISCHRG");
(FORCED_SHUTDOWN, "FSD");
(LOW_BATTERY,     "LB");
(NOCOMM,          "NOCOMM");
(OFFLINE,         "OFF");
(ONLINE,          "OL");
(ON_BATTERY,      "OB");
(OVERLOADED,      "OVER");
(REPLACE_BATTERY, "RB");
(TEST,            "TEST");
(TICK,            "TICK");
(TOCK,            "TOCK");
(TRIM,            "TRIM");
);

impl std::fmt::Display for UpsStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.len() > 1 {
      let parts: Vec<&str> = self.iter().map(|v| get_state_str(v)).collect();
      f.write_str(&parts.join(" "))
    } else {
      f.write_str(get_state_str(*self))
    }
  }
}

impl std::str::FromStr for UpsStatus {
  type Err = UnsupportedStatusError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::try_from(s)
  }
}

impl Default for UpsStatus {
  #[inline]
  fn default() -> Self {
    Self(0)
  }
}

impl UpsStatus {
  #[inline]
  pub const fn has(&self, rhs: UpsStatus) -> bool {
    (self.0 & rhs.0) == rhs.0
  }

  #[inline]
  pub const fn set(self, value: UpsStatus) -> Self {
    Self(self.0 | value.0)
  }

  #[inline]
  pub const fn unset(self, value: UpsStatus) -> Self {
    Self(self.0 & (!value.0))
  }

  #[inline]
  pub const fn len(&self) -> u32 {
    self.0.count_ones()
  }

  pub fn iter(&self) -> Iter {
    Iter {
      state: self.clone(),
    }
  }
}

impl core::ops::BitOr for UpsStatus {
  type Output = UpsStatus;

  #[inline]
  fn bitor(self, rhs: Self) -> Self::Output {
    UpsStatus(self.0 | rhs.0)
  }
}

impl core::ops::BitAnd for UpsStatus {
  type Output = UpsStatus;

  #[inline]
  fn bitand(self, rhs: Self) -> Self::Output {
    UpsStatus(self.0 & rhs.0)
  }
}

impl core::ops::BitXor for UpsStatus {
  type Output = UpsStatus;

  #[inline]
  fn bitxor(self, rhs: Self) -> Self::Output {
    UpsStatus(self.0 ^ rhs.0)
  }
}

impl core::ops::BitOrAssign for UpsStatus {
  #[inline]
  fn bitor_assign(&mut self, rhs: Self) {
    self.0 |= rhs.0;
  }
}

impl core::ops::BitAndAssign for UpsStatus {
  #[inline]
  fn bitand_assign(&mut self, rhs: Self) {
    self.0 &= rhs.0;
  }
}

impl core::ops::BitXorAssign for UpsStatus {
  #[inline]
  fn bitxor_assign(&mut self, rhs: Self) {
    self.0 ^= rhs.0;
  }
}

pub struct Iter {
  state: UpsStatus,
}

impl Iterator for Iter {
  type Item = UpsStatus;

  fn next(&mut self) -> Option<Self::Item> {
    if self.state.0 == 0 {
      None
    } else {
      let mask: u32 = 1 << self.state.0.trailing_zeros();
      let yield_val = UpsStatus(self.state.0 & mask);
      self.state = self.state.unset(yield_val);

      Some(yield_val)
    }
  }
}

impl IntoIterator for UpsStatus {
  type Item = UpsStatus;
  type IntoIter = Iter;

  fn into_iter(self) -> Self::IntoIter {
    Self::IntoIter { state: self }
  }
}
