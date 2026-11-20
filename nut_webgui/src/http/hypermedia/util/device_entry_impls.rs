use super::{
  semantic_type::SemanticType,
  unit::{
    ApparentPower, Approx, Celcius, OneOf, Percentage, RealPower, Second, UnitDisplay, Voltage,
  },
  ups_status::StatusDetailIter,
};
use crate::state::DeviceEntry;
use askama::PrimitiveType;
use upsmc::{Value, VarName};

// Provides hypermedia specific impls for DeviceEntry struct
impl DeviceEntry {
  #[inline]
  pub fn get_beeper_status(&self) -> Option<bool> {
    match self.variables.get(VarName::UPS_BEEPER_STATUS) {
      Some(Value::String(v)) => Some(v.eq_ignore_ascii_case("enabled")),
      _ => None,
    }
  }

  pub fn get_ups_load(&self) -> Option<Percentage> {
    let mut value = Percentage::try_from(self.variables.get(VarName::UPS_LOAD)?).ok()?;
    value.set_semantic_type(SemanticType::from_upper_limits(value.get(), 50.0, 75.0));
    Some(value)
  }

  pub fn get_battery_charge(&self) -> Option<Percentage> {
    let mut value = Percentage::try_from(self.variables.get(VarName::BATTERY_CHARGE)?).ok()?;
    let warn_level = self
      .variables
      .get(VarName::BATTERY_CHARGE_WARNING)
      .and_then(|v| v.as_lossy_f64())
      .unwrap_or(50.0);
    let danger_level = self
      .variables
      .get(VarName::BATTERY_CHARGE_LOW)
      .and_then(|v| v.as_lossy_f64())
      .unwrap_or(25.0);

    value.set_semantic_type(SemanticType::from_lower_limits(
      value.get(),
      danger_level,
      warn_level,
    ));

    Some(value)
  }

  pub fn get_battery_runtime(&self) -> Option<Second> {
    let mut value = Second::try_from(self.variables.get(VarName::BATTERY_RUNTIME)?).ok()?;
    let danger_level = self
      .variables
      .get(VarName::BATTERY_RUNTIME_LOW)
      .and_then(|v| v.as_lossy_i64())
      .unwrap_or(60);

    if value.get() <= danger_level {
      value.set_semantic_type(SemanticType::Error);
    } else {
      value.set_semantic_type(SemanticType::Success);
    }

    Some(value)
  }

  pub fn get_real_power(&self) -> Option<OneOf<RealPower, Approx<RealPower>>> {
    let power = self
      .variables
      .get(VarName::UPS_REALPOWER)
      .map(|v| RealPower::try_from(v).ok())
      .flatten();
    let load = self
      .variables
      .get(VarName::UPS_LOAD)
      .and_then(|v| v.as_lossy_f64());

    let mut unit = match power {
      Some(v) => OneOf::T1(v),
      None => {
        let load = load?;
        let nominal = self
          .variables
          .get(VarName::UPS_REALPOWER_NOMINAL)
          .and_then(|v| v.as_lossy_f64())?;
        let calc = (nominal * load / 100.0).round();

        OneOf::T2(RealPower::from(calc).into())
      }
    };

    if let Some(load) = load {
      unit.set_semantic_type(SemanticType::from_upper_limits(load.get(), 50.0, 75.0));
    }

    Some(unit)
  }

  pub fn get_apparent_power(&self) -> Option<OneOf<ApparentPower, Approx<ApparentPower>>> {
    let power = self
      .variables
      .get(VarName::UPS_POWER)
      .map(|v| ApparentPower::try_from(v).ok())
      .flatten();
    let load = self
      .variables
      .get(VarName::UPS_LOAD)
      .and_then(|v| v.as_lossy_f64());

    let mut unit = match power {
      Some(v) => OneOf::T1(v),
      None => {
        let load = load?;
        let nominal = self
          .variables
          .get(VarName::UPS_POWER_NOMINAL)
          .and_then(|v| v.as_lossy_f64())?;
        let calc = (nominal * load / 100.0).round();

        OneOf::T2(ApparentPower::from(calc).into())
      }
    };

    if let Some(load) = load {
      unit.set_semantic_type(SemanticType::from_upper_limits(load.get(), 50.0, 75.0));
    }

    Some(unit)
  }

  pub fn get_input_voltage(&self) -> Option<Voltage> {
    let mut value = Voltage::try_from(self.variables.get(VarName::INPUT_VOLTAGE)?).ok()?;
    let low = self
      .variables
      .get(VarName::INPUT_VOLTAGE_LOW_CRITICAL)
      .and_then(|v| v.as_lossy_f64());
    let high = self
      .variables
      .get(VarName::INPUT_VOLTAGE_HIGH_CRITICAL)
      .and_then(|v| v.as_lossy_f64());

    if let (Some(min), Some(max)) = (low, high) {
      value.set_semantic_type(SemanticType::from_bounds(value.get(), min, max));
    }

    Some(value)
  }

  pub fn get_output_voltage(&self) -> Option<Voltage> {
    let mut value = Voltage::try_from(self.variables.get(VarName::OUTPUT_VOLTAGE)?).ok()?;
    let low = self
      .variables
      .get(VarName::OUTPUT_VOLTAGE_LOW)
      .and_then(|v| v.as_lossy_f64());
    let high = self
      .variables
      .get(VarName::OUTPUT_VOLTAGE_HIGH)
      .and_then(|v| v.as_lossy_f64());

    if let (Some(min), Some(max)) = (low, high) {
      value.set_semantic_type(SemanticType::from_bounds(value.get(), min, max));
    }

    Some(value)
  }

  pub fn get_battery_voltage(&self) -> Option<Voltage> {
    let mut value = Voltage::try_from(self.variables.get(VarName::BATTERY_VOLTAGE)?).ok()?;
    let low = self
      .variables
      .get(VarName::BATTERY_VOLTAGE_LOW)
      .and_then(|v| v.as_lossy_f64());
    let high = self
      .variables
      .get(VarName::BATTERY_VOLTAGE_HIGH)
      .and_then(|v| v.as_lossy_f64());

    if let (Some(min), Some(max)) = (low, high) {
      value.set_semantic_type(SemanticType::from_bounds(value.get(), min, max));
    }

    Some(value)
  }

  pub fn get_battery_temperature(&self) -> Option<Celcius> {
    let mut value = Celcius::try_from(self.variables.get(VarName::BATTERY_TEMPERATURE)?).ok()?;
    value.set_semantic_type(SemanticType::from_bounds(value.get(), 0.0, 50.0));
    Some(value)
  }

  pub fn get_ambient_temperature(&self) -> Option<Celcius> {
    let mut value = Celcius::try_from(self.variables.get(VarName::AMBIENT_TEMPERATURE)?).ok()?;
    let low = self
      .variables
      .get(VarName::AMBIENT_TEMPERATURE_LOW)
      .and_then(|v| v.as_lossy_f64())
      .unwrap_or(0.0);
    let high = self
      .variables
      .get(VarName::AMBIENT_TEMPERATURE_HIGH)
      .and_then(|v| v.as_lossy_f64())
      .unwrap_or(50.0);

    value.set_semantic_type(SemanticType::from_bounds(value.get(), low, high));
    Some(value)
  }

  /// ups temperature (e.g. heatsink sensors)
  pub fn get_ups_temperature(&self) -> Option<Celcius> {
    let mut value = Celcius::try_from(self.variables.get(VarName::UPS_TEMPERATURE)?).ok()?;
    let low = self
      .variables
      .get(VarName::UPS_TEMPERATURE_LOW)
      .and_then(|v| v.as_lossy_f64())
      .unwrap_or(0.0);
    let high = self
      .variables
      .get(VarName::UPS_TEMPERATURE_HIGH)
      .and_then(|v| v.as_lossy_f64())
      .unwrap_or(50.0);

    value.set_semantic_type(SemanticType::from_bounds(value.get(), low, high));
    Some(value)
  }

  #[inline]
  pub const fn iter_status(&self) -> StatusDetailIter {
    StatusDetailIter::new(self.status)
  }
}
