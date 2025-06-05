use super::semantic_classes::SemanticType;
use crate::device_entry::DeviceEntry;
use askama::FastWritable;
use core::ops::Deref;
use nut_webgui_upsmc::{Value, VarName};
use std::{borrow::Cow, fmt::Write};

#[derive(Debug)]
pub struct ValueDetail<'a> {
  pub value: Cow<'a, Value>,
  pub class: SemanticType,
  pub unit_sign: Option<char>,
}

// Provides hypermedia specific impls for DeviceEntry struct
impl DeviceEntry {
  pub fn get_ups_temperature(&self) -> Option<ValueDetail<'_>> {
    let temperature = self.variables.get(VarName::UPS_TEMPERATURE)?;
    let unit_sign = if temperature.is_text() {
      None
    } else {
      Some('℃')
    };

    let semantic_class = {
      match temperature.as_lossly_i64() {
        Some(v) => {
          let low_temp = self
            .variables
            .get(VarName::UPS_TEMPERATURE_LOW)
            .map(|v| v.as_lossly_i64())
            .flatten()
            .unwrap_or(0);

          let high_temp = self
            .variables
            .get(VarName::UPS_TEMPERATURE_HIGH)
            .map(|v| v.as_lossly_i64())
            .flatten()
            .unwrap_or(60);

          if v <= low_temp || v >= high_temp {
            SemanticType::Error
          } else {
            SemanticType::Success
          }
        }
        _ => SemanticType::Info,
      }
    };

    Some(ValueDetail {
      value: Cow::Borrowed(temperature),
      class: semantic_class,
      unit_sign,
    })
  }

  pub fn get_ups_load(&self) -> Option<ValueDetail<'_>> {
    let load = self.variables.get(VarName::UPS_LOAD)?;
    let unit_sign = if load.is_text() { None } else { Some('%') };

    let semantic_class = {
      match load.as_lossly_i64() {
        Some(v) => SemanticType::from_range(v, 45, 75),
        _ => SemanticType::Info,
      }
    };

    Some(ValueDetail {
      value: Cow::Borrowed(load),
      class: semantic_class,
      unit_sign,
    })
  }

  pub fn get_battery_charge(&self) -> Option<ValueDetail<'_>> {
    let charge = self.variables.get(VarName::BATTERY_CHARGE)?;
    let unit_sign = if charge.is_text() { None } else { Some('%') };

    let semantic_class = {
      let warn_level = self
        .variables
        .get(VarName::BATTERY_CHARGE_WARNING)
        .map(|v| v.as_lossly_i64())
        .flatten()
        .unwrap_or(50);

      let danger_level = self
        .variables
        .get(VarName::BATTERY_CHARGE_LOW)
        .map(|v| v.as_lossly_i64())
        .flatten()
        .unwrap_or(25);

      match charge.as_lossly_i64() {
        Some(v) => SemanticType::from_range_inverted(v, danger_level, warn_level),
        _ => SemanticType::Info,
      }
    };

    Some(ValueDetail {
      value: Cow::Borrowed(charge),
      class: semantic_class,
      unit_sign,
    })
  }

  pub fn get_battery_runtime(&self) -> Option<ValueDetail<'_>> {
    let battery_runtime = self.variables.get(VarName::BATTERY_RUNTIME)?;

    let semantic_class = {
      let danger_level = self
        .variables
        .get(VarName::BATTERY_RUNTIME_LOW)
        .map(|v| v.as_lossly_i64())
        .flatten()
        .unwrap_or(60);

      match battery_runtime.as_lossly_i64() {
        Some(v) => {
          if v < danger_level {
            SemanticType::Error
          } else {
            SemanticType::Success
          }
        }
        _ => SemanticType::Info,
      }
    };

    Some(ValueDetail {
      value: Cow::Borrowed(battery_runtime),
      class: semantic_class,
      unit_sign: None,
    })
  }

  pub fn get_real_power(&self) -> Option<ValueDetail<'_>> {
    let realpower = {
      match self
        .variables
        .get(VarName::UPS_REALPOWER)
        .map(|v| v.as_lossly_f64())
      {
        Some(v) => v,
        None => {
          let load = self
            .variables
            .get(VarName::UPS_LOAD)
            .map(|v| v.as_lossly_f64())
            .flatten();

          let nominal_realpower = self
            .variables
            .get(VarName::UPS_REALPOWER_NOMINAL)
            .map(|v| v.as_lossly_f64())
            .flatten();

          match (load, nominal_realpower) {
            (Some(load), Some(nominal_realpower)) => Some((nominal_realpower * load) / 100.0f64),
            _ => None,
          }
        }
      }
    }?;

    Some(ValueDetail {
      value: Cow::Owned(Value::from(realpower)),
      class: SemanticType::Info,
      unit_sign: Some('W'),
    })
  }

  pub fn get_apparent_power(&self) -> Option<ValueDetail<'_>> {
    let power = {
      match self
        .variables
        .get(VarName::UPS_POWER)
        .map(|v| v.as_lossly_f64())
      {
        Some(v) => v,
        None => {
          let load = self
            .variables
            .get(VarName::UPS_LOAD)
            .map(|v| v.as_lossly_f64())
            .flatten();

          let nominal_power = self
            .variables
            .get(VarName::UPS_POWER_NOMINAL)
            .map(|v| v.as_lossly_f64())
            .flatten();

          match (load, nominal_power) {
            (Some(load), Some(nominal_power)) => Some((nominal_power * load) / 100.0f64),
            _ => None,
          }
        }
      }
    }?;

    Some(ValueDetail {
      value: Cow::Owned(Value::from(power)),
      class: SemanticType::Info,
      unit_sign: Some('A'),
    })
  }

  /// Returns fist matching ups power information
  ///
  /// Priority order:
  /// - ups.realpower
  /// - ups.power
  /// - calculated realpower based on ups.realpower.nominal
  /// - calculated apparent power based on ups.power.nominal
  pub fn get_power(&self) -> Option<ValueDetail<'_>> {
    if self.variables.contains_key(VarName::UPS_REALPOWER) {
      self.get_real_power()
    } else if self.variables.contains_key(VarName::UPS_POWER) {
      self.get_apparent_power()
    } else {
      // Fallback to calculted values
      self.get_real_power().or_else(|| self.get_apparent_power())
    }
  }
}

impl Deref for ValueDetail<'_> {
  type Target = Value;

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.value.deref()
  }
}

impl std::fmt::Display for ValueDetail<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.value.fmt(f)?;

    if let Some(sign) = self.unit_sign {
      f.write_char(sign)?;
    }

    Ok(())
  }
}

impl FastWritable for ValueDetail<'_> {
  fn write_into<W: core::fmt::Write + ?Sized>(
    &self,
    dest: &mut W,
    _: &dyn askama::Values,
  ) -> askama::Result<()> {
    dest.write_str(self.value.as_str().as_ref())?;

    if let Some(sign) = self.unit_sign {
      dest.write_char(sign)?;
    }

    Ok(())
  }
}
