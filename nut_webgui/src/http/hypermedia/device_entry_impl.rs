use crate::{
  device_entry::DeviceEntry,
  http::hypermedia::{
    semantic_type::SemanticType,
    unit::{
      ApparentPower, Approx, Celcius, OneOf, Percentage, RealPower, RemainingSeconds, UnitDisplay,
    },
  },
};
use nut_webgui_upsmc::{Value, VarName};

use super::unit::Voltage;

// Provides hypermedia specific impls for DeviceEntry struct
impl DeviceEntry {
  #[inline]
  pub fn get_status(&self) -> Option<&str> {
    match self.variables.get(VarName::UPS_STATUS) {
      Some(Value::String(v)) => Some(v.as_ref()),
      _ => None,
    }
  }

  #[inline]
  pub fn get_beeper_status(&self) -> Option<bool> {
    match self.variables.get(VarName::UPS_BEEPER_STATUS) {
      Some(Value::String(v)) => Some(v.eq_ignore_ascii_case("enabled")),
      _ => None,
    }
  }

  pub fn get_ups_temperature(&self) -> Option<Celcius> {
    let temp_var = self.variables.get(VarName::UPS_TEMPERATURE)?;

    match Celcius::try_from(temp_var) {
      Ok(mut value) => {
        let low_temp = self
          .variables
          .get(VarName::UPS_TEMPERATURE_LOW)
          .and_then(|v| v.as_lossly_f64())
          .unwrap_or(0.0);

        let high_temp = self
          .variables
          .get(VarName::UPS_TEMPERATURE_HIGH)
          .and_then(|v| v.as_lossly_f64())
          .unwrap_or(60.0);

        value.set_semantic_type(SemanticType::from_range(
          value.as_raw_value(),
          low_temp,
          high_temp,
        ));

        Some(value)
      }
      Err(_) => None,
    }
  }

  pub fn get_ups_load(&self) -> Option<Percentage> {
    let load_var = self.variables.get(VarName::UPS_LOAD)?;

    match Percentage::try_from(load_var) {
      Ok(mut value) => {
        value.set_semantic_type(SemanticType::from_range(value.as_raw_value(), 45.0, 75.0));

        Some(value)
      }
      Err(_) => None,
    }
  }

  pub fn get_battery_charge(&self) -> Option<Percentage> {
    let charge_var = self.variables.get(VarName::BATTERY_CHARGE)?;

    match Percentage::try_from(charge_var) {
      Ok(mut value) => {
        let warn_level = self
          .variables
          .get(VarName::BATTERY_CHARGE_WARNING)
          .and_then(|v| v.as_lossly_f64())
          .unwrap_or(50.0);

        let danger_level = self
          .variables
          .get(VarName::BATTERY_CHARGE_LOW)
          .and_then(|v| v.as_lossly_f64())
          .unwrap_or(25.0);

        value.set_semantic_type(SemanticType::from_range_inverted(
          value.as_raw_value(),
          danger_level,
          warn_level,
        ));

        Some(value)
      }
      Err(_) => None,
    }
  }

  pub fn get_battery_runtime(&self) -> Option<RemainingSeconds> {
    let battery_runtime = self.variables.get(VarName::BATTERY_RUNTIME)?;

    match RemainingSeconds::try_from(battery_runtime) {
      Ok(mut value) => {
        let danger_level = self
          .variables
          .get(VarName::BATTERY_RUNTIME_LOW)
          .and_then(|v| v.as_lossly_i64())
          .unwrap_or(60);

        if value.as_raw_value() < danger_level {
          value.set_semantic_type(SemanticType::Error);
        } else {
          value.set_semantic_type(SemanticType::Success);
        }

        Some(value)
      }
      Err(_) => None,
    }
  }

  fn get_approx_real_power(&self) -> Option<Approx<RealPower>> {
    let load = self
      .variables
      .get(VarName::UPS_LOAD)
      .and_then(|v| v.as_lossly_f64())?;

    let nominal_power = self
      .variables
      .get(VarName::UPS_REALPOWER_NOMINAL)
      .and_then(|v| v.as_lossly_f64())?;

    let calc = (nominal_power * load / 100.0).round();

    Some(RealPower::from(calc).into())
  }

  fn get_approx_apparent_power(&self) -> Option<Approx<ApparentPower>> {
    let load = self
      .variables
      .get(VarName::UPS_LOAD)
      .and_then(|v| v.as_lossly_f64())?;

    let nominal = self
      .variables
      .get(VarName::UPS_POWER_NOMINAL)
      .and_then(|v| v.as_lossly_f64())?;

    let calc = (nominal * load / 100.0).round();

    Some(ApparentPower::from(calc).into())
  }

  pub fn get_real_power(&self) -> Option<OneOf<RealPower, Approx<RealPower>>> {
    if let Some(real_power) = self.variables.get(VarName::UPS_REALPOWER)
      && let Ok(value) = RealPower::try_from(real_power)
    {
      Some(OneOf::T1(value))
    } else {
      self.get_approx_real_power().map(|v| OneOf::T2(v))
    }
  }

  pub fn get_apparent_power(&self) -> Option<OneOf<ApparentPower, Approx<ApparentPower>>> {
    if let Some(power) = self.variables.get(VarName::UPS_POWER)
      && let Ok(value) = ApparentPower::try_from(power)
    {
      Some(OneOf::T1(value))
    } else {
      self.get_approx_apparent_power().map(|v| OneOf::T2(v))
    }
  }

  pub fn get_input_voltage(&self) -> Option<Voltage> {
    let volt_var = self.variables.get(VarName::INPUT_VOLTAGE)?;

    match Voltage::try_from(volt_var) {
      Ok(mut value) => {
        let low_volt = self
          .variables
          .get(VarName::INPUT_VOLTAGE_LOW_CRITICAL)
          .and_then(|v| v.as_lossly_f64());

        let high_volt = self
          .variables
          .get(VarName::INPUT_VOLTAGE_HIGH_CRITICAL)
          .and_then(|v| v.as_lossly_f64());

        let semantic_type = if low_volt.is_some_and(|target| value.as_raw_value() <= target)
          || high_volt.is_some_and(|target| value.as_raw_value() >= target)
        {
          SemanticType::Warning
        } else {
          SemanticType::Success
        };

        value.set_semantic_type(semantic_type);

        Some(value)
      }
      Err(_) => None,
    }
  }

  pub fn get_output_voltage(&self) -> Option<Voltage> {
    let volt_var = self.variables.get(VarName::OUTPUT_VOLTAGE)?;

    match Voltage::try_from(volt_var) {
      Ok(mut value) => {
        let low_volt = self
          .variables
          .get(VarName::OUTPUT_VOLTAGE_LOW)
          .and_then(|v| v.as_lossly_f64());

        let high_volt = self
          .variables
          .get(VarName::OUTPUT_VOLTAGE_HIGH)
          .and_then(|v| v.as_lossly_f64());

        let semantic_type = if low_volt.is_some_and(|target| value.as_raw_value() <= target)
          || high_volt.is_some_and(|target| value.as_raw_value() >= target)
        {
          SemanticType::Error
        } else {
          SemanticType::Success
        };

        value.set_semantic_type(semantic_type);

        Some(value)
      }
      Err(_) => None,
    }
  }

  pub fn get_battery_voltage(&self) -> Option<Voltage> {
    let volt_var = self.variables.get(VarName::BATTERY_VOLTAGE)?;

    match Voltage::try_from(volt_var) {
      Ok(mut value) => {
        let low_volt = self
          .variables
          .get(VarName::BATTERY_VOLTAGE_LOW)
          .and_then(|v| v.as_lossly_f64());

        let high_volt = self
          .variables
          .get(VarName::BATTERY_VOLTAGE_HIGH)
          .and_then(|v| v.as_lossly_f64());

        let semantic_type = if low_volt.is_some_and(|target| value.as_raw_value() <= target)
          || high_volt.is_some_and(|target| value.as_raw_value() >= target)
        {
          SemanticType::Error
        } else {
          SemanticType::Success
        };

        value.set_semantic_type(semantic_type);

        Some(value)
      }
      Err(_) => None,
    }
  }
}
