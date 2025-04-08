use super::internal::{ReadOnlyStr, Repr, ascii_rules::NutAsciiText};
use crate::errors::ParseErrors;

macro_rules! impl_standard_names {
  ($enum_name:ident,
  $(
    $(#[$docs:meta])*
    ($const_name:ident, $variant_name:ident, $value:literal);
  )+
  ) => {
    #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
    enum $enum_name {
      $( $variant_name,)+
    }

    impl $enum_name {
      pub const fn as_str(&self) -> &'static str {
        match self {
          $(Self::$variant_name => $value,)+
        }
      }
    }

    impl AsRef<str> for $enum_name {
       #[inline]
       fn as_ref(&self) -> &str {
        self.as_str()
      }
    }

    impl std::fmt::Display for $enum_name {
       #[inline]
       fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
      }
    }

    impl TryFrom<&str> for $enum_name {
      type Error = ();

      fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
          $($value => Ok(Self::$variant_name),)+
          _ => Err(()),
        }
      }
    }

    impl VarName {
      $(
        $(#[$docs])*
        pub const $const_name: $crate::var_name::VarName = $crate::var_name::VarName {
          name: $crate::internal::Repr::Standard($enum_name::$variant_name)
        };
      )+
    }
  };
}

impl_standard_names!(
  StandardNames,
  (AMBIENT_HUMIDITY, AmbientHumidity, "ambient.humidity");
  (AMBIENT_HUMIDITY_ALARM, AmbientHumidityAlarm, "ambient.humidity.alarm");
  (AMBIENT_HUMIDITY_ALARM_ENABLE, AmbientHumidityAlarmEnable, "ambient.humidity.alarm.enable");
  (AMBIENT_HUMIDITY_ALARM_MAXIMUM, AmbientHumidityAlarmMaximum, "ambient.humidity.alarm.maximum");
  (AMBIENT_HUMIDITY_ALARM_MINIMUM, AmbientHumidityAlarmMinimum, "ambient.humidity.alarm.minimum");
  (AMBIENT_HUMIDITY_HIGH, AmbientHumidityHigh, "ambient.humidity.high");
  (AMBIENT_HUMIDITY_HIGH_CRITICAL, AmbientHumidityHighCritical, "ambient.humidity.high.critical");
  (AMBIENT_HUMIDITY_HIGH_WARNING, AmbientHumidityHighWarning, "ambient.humidity.high.warning");
  (AMBIENT_HUMIDITY_LOW, AmbientHumidityLow, "ambient.humidity.low");
  (AMBIENT_HUMIDITY_LOW_CRITICAL, AmbientHumidityLowCritical, "ambient.humidity.low.critical");
  (AMBIENT_HUMIDITY_LOW_WARNING, AmbientHumidityLowWarning, "ambient.humidity.low.warning");
  (AMBIENT_HUMIDITY_STATUS, AmbientHumidityStatus, "ambient.humidity.status");
  (AMBIENT_PRESENT, AmbientPresent, "ambient.present");
  (AMBIENT_TEMPERATURE, AmbientTemperature, "ambient.temperature");
  (AMBIENT_TEMPERATURE_ALARM, AmbientTemperatureAlarm, "ambient.temperature.alarm");
  (AMBIENT_TEMPERATURE_ALARM_ENABLE, AmbientTemperatureAlarmEnable, "ambient.temperature.alarm.enable");
  (AMBIENT_TEMPERATURE_ALARM_MAXIMUM, AmbientTemperatureAlarmMaximum, "ambient.temperature.alarm.maximum");
  (AMBIENT_TEMPERATURE_ALARM_MINIMUM, AmbientTemperatureAlarmMinimum, "ambient.temperature.alarm.minimum");
  (AMBIENT_TEMPERATURE_HIGH, AmbientTemperatureHigh, "ambient.temperature.high");
  (AMBIENT_TEMPERATURE_HIGH_CRITICAL, AmbientTemperatureHighCritical, "ambient.temperature.high.critical");
  (AMBIENT_TEMPERATURE_HIGH_WARNING, AmbientTemperatureHighWarning, "ambient.temperature.high.warning");
  (AMBIENT_TEMPERATURE_LOW, AmbientTemperatureLow, "ambient.temperature.low");
  (AMBIENT_TEMPERATURE_LOW_CRITICAL, AmbientTemperatureLowCritical, "ambient.temperature.low.critical");
  (AMBIENT_TEMPERATURE_LOW_WARNING, AmbientTemperatureLowWarning, "ambient.temperature.low.warning");
  (AMBIENT_TEMPERATURE_STATUS, AmbientTemperatureStatus, "ambient.temperature.status");
  (BATTERY_CAPACITY, BatteryCapacity, "battery.capacity");
  (BATTERY_CHARGE, BatteryCharge, "battery.charge");
  (BATTERY_CHARGE_APPROX, BatteryChargeApprox, "battery.charge.approx");
  (BATTERY_CHARGE_LOW, BatteryChargeLow, "battery.charge.low");
  (BATTERY_CHARGE_RESTART, BatteryChargeRestart, "battery.charge.restart");
  (BATTERY_CHARGE_WARNING, BatteryChargeWarning, "battery.charge.warning");
  (BATTERY_CURRENT, BatteryCurrent, "battery.current");
  (BATTERY_ENERGYSAVE, BatteryEnergysave, "battery.energysave");
  (BATTERY_ENERGYSAVE_DELAY, BatteryEnergysaveDelay, "battery.energysave.delay");
  (BATTERY_ENERGYSAVE_LOAD, BatteryEnergysaveLoad, "battery.energysave.load");
  (BATTERY_ENERGYSAVE_REALPOWER, BatteryEnergysaveRealpower, "battery.energysave.realpower");
  (BATTERY_PROTECTION, BatteryProtection, "battery.protection");
  (BATTERY_RUNTIME, BatteryRuntime, "battery.runtime");
  (BATTERY_RUNTIME_LOW, BatteryRuntimeLow, "battery.runtime.low");
  (BATTERY_TEMPERATURE, BatteryTemperature, "battery.temperature");
  (BATTERY_TYPE, BatteryType, "battery.type");
  (BATTERY_VOLTAGE, BatteryVoltage, "battery.voltage");
  (BATTERY_VOLTAGE_NOMINAL, BatteryVoltageNominal, "battery.voltage.nominal");
  (DEVICE_MFR, DeviceMfr, "device.mfr");
  (DEVICE_MODEL, DeviceModel, "device.model");
  (DEVICE_SERIAL, DeviceSerial, "device.serial");
  (DEVICE_TYPE, DeviceType, "device.type");
  (DRIVER_NAME, DriverName, "driver.name");
  (DRIVER_PARAMETER_LOWBATT, DriverParameterLowbatt, "driver.parameter.lowbatt");
  (DRIVER_PARAMETER_OFFDELAY, DriverParameterOffdelay, "driver.parameter.offdelay");
  (DRIVER_PARAMETER_ONDELAY, DriverParameterOndelay, "driver.parameter.ondelay");
  (DRIVER_PARAMETER_POLLFREQ, DriverParameterPollfreq, "driver.parameter.pollfreq");
  (DRIVER_PARAMETER_POLLINTERVAL, DriverParameterPollinterval, "driver.parameter.pollinterval");
  (DRIVER_PARAMETER_PORT, DriverParameterPort, "driver.parameter.port");
  (DRIVER_PARAMETER_SYNCHRONOUS, DriverParameterSynchronous, "driver.parameter.synchronous");
  (DRIVER_PARAMETER_VENDORID, DriverParameterVendorid, "driver.parameter.vendorid");
  (DRIVER_VERSION, DriverVersion, "driver.version");
  (DRIVER_VERSION_DATA, DriverVersionData, "driver.version.data");
  (DRIVER_VERSION_INTERNAL, DriverVersionInternal, "driver.version.internal");
  (INPUT_CURRENT, InputCurrent, "input.current");
  (INPUT_CURRENT_NOMINAL, InputCurrentNominal, "input.current.nominal");
  (INPUT_CURRENT_STATUS, InputCurrentStatus, "input.current.status");
  (INPUT_FREQUENCY, InputFrequency, "input.frequency");
  (INPUT_FREQUENCY_EXTENDED, InputFrequencyExtended, "input.frequency.extended");
  (INPUT_FREQUENCY_HIGH, InputFrequencyHigh, "input.frequency.high");
  (INPUT_FREQUENCY_LOW, InputFrequencyLow, "input.frequency.low");
  (INPUT_FREQUENCY_NOMINAL, InputFrequencyNominal, "input.frequency.nominal");
  (INPUT_FREQUENCY_STATUS, InputFrequencyStatus, "input.frequency.status");
  (INPUT_LOAD, InputLoad, "input.load");
  (INPUT_POWER, InputPower, "input.power");
  (INPUT_QUALITY, InputQuality, "input.quality");
  (INPUT_REALPOWER, InputRealpower, "input.realpower");
  (INPUT_SENSITIVITY, InputSensitivity, "input.sensitivity");
  (INPUT_SOURCE, InputSource, "input.source");
  (INPUT_SOURCE_PREFERRED, InputSourcePreferred, "input.source.preferred");
  (INPUT_TRANSFER_BOOST_HIGH, InputTransferBoostHigh, "input.transfer.boost.high");
  (INPUT_TRANSFER_BOOST_LOW, InputTransferBoostLow, "input.transfer.boost.low");
  (INPUT_TRANSFER_DELAY, InputTransferDelay, "input.transfer.delay");
  (INPUT_TRANSFER_HIGH, InputTransferHigh, "input.transfer.high");
  (INPUT_TRANSFER_LOW, InputTransferLow, "input.transfer.low");
  (INPUT_TRANSFER_TRIM_HIGH, InputTransferTrimHigh, "input.transfer.trim.high");
  (INPUT_TRANSFER_TRIM_LOW, InputTransferTrimLow, "input.transfer.trim.low");
  (INPUT_VOLTAGE, InputVoltage, "input.voltage");
  (INPUT_VOLTAGE_NOMINAL, InputVoltageNominal, "input.voltage.nominal");
  (OUTPUT_CURRENT, OutputCurrent, "output.current");
  (OUTPUT_CURRENT_NOMINAL, OutputCurrentNominal, "output.current.nominal");
  (OUTPUT_FREQUENCY, OutputFrequency, "output.frequency");
  (OUTPUT_FREQUENCY_NOMINAL, OutputFrequencyNominal, "output.frequency.nominal");
  (OUTPUT_VOLTAGE, OutputVoltage, "output.voltage");
  (OUTPUT_VOLTAGE_NOMINAL, OutputVoltageNominal, "output.voltage.nominal");
  (UPS_BEEPER_STATUS, UpsBeeperStatus, "ups.beeper.status");
  (UPS_CONTACTS, UpsContacts, "ups.contacts");
  (UPS_DELAY_SHUTDOWN, UpsDelayShutdown, "ups.delay.shutdown");
  (UPS_DELAY_START, UpsDelayStart, "ups.delay.start");
  (UPS_FIRMWARE, UpsFirmware, "ups.firmware");
  (UPS_LOAD, UpsLoad, "ups.load");
  (UPS_MFR, UpsMfr, "ups.mfr");
  (UPS_MODEL, UpsModel, "ups.model");
  (UPS_POWER, UpsPower, "ups.power");
  (UPS_POWER_NOMINAL, UpsPowerNominal, "ups.power.nominal");
  (UPS_PRODUCTID, UpsProductid, "ups.productid");
  (UPS_REALPOWER, UpsRealpower, "ups.realpower");
  (UPS_REALPOWER_NOMINAL, UpsRealpowerNominal, "ups.realpower.nominal");
  (UPS_SERIAL, UpsSerial, "ups.serial");
  (UPS_STATUS, UpsStatus, "ups.status");
  (UPS_TEMPERATURE, UpsTemperature, "ups.temperature");
  (UPS_TEST_INTERVAL, UpsTestInterval, "ups.test.interval");
  (UPS_TEST_RESULT, UpsTestResult, "ups.test.result");
  (UPS_TIMER_SHUTDOWN, UpsTimerShutdown, "ups.timer.shutdown");
  (UPS_TIMER_START, UpsTimerStart, "ups.timer.start");
  (UPS_VENDORID, UpsVendorid, "ups.vendorid");
);

/// Checks if [`&str`] matches to varname ABNF grammar.
///
/// ```abnf
/// varname = 1*LOWERCASE_ASCII *62( DOT 1*(DIGIT / LOWERCASE_ASCII) )
/// ```
fn is_var_name<T>(name: T) -> Result<(), ParseErrors>
where
  T: AsRef<str>,
{
  let name = name.as_ref().as_bytes();

  if name.is_empty() {
    Err(ParseErrors::Empty)
  } else if let Some(b'.') = name.get(0) {
    Err(ParseErrors::InvalidChar { position: 0 })
  } else {
    for (idx, byte) in name.iter().enumerate() {
      if !byte.is_ascii_nut_var() {
        return Err(ParseErrors::InvalidChar { position: idx });
      }
    }

    Ok(())
  }
}

/// UPS variable name.
///
/// ```abnf
/// varname = 1*LOWERCASE_ASCII *62( DOT 1*(DIGIT / LOWERCASE_ASCII) )
/// ```
#[derive(Debug, Clone, Eq, Hash)]
pub struct VarName {
  name: Repr<StandardNames, ReadOnlyStr>,
}

impl VarName {
  pub fn new<T>(name: T) -> Result<Self, ParseErrors>
  where
    T: AsRef<str>,
  {
    is_var_name(&name)?;

    Ok(Self::new_unchecked(name))
  }

  pub fn new_unchecked<T>(name: T) -> Self
  where
    T: AsRef<str>,
  {
    let name_str: &str = name.as_ref();

    if let Ok(name) = StandardNames::try_from(name_str) {
      Self {
        name: Repr::Standard(name),
      }
    } else {
      Self {
        name: Repr::Custom(ReadOnlyStr::from(name_str)),
      }
    }
  }

  #[inline]
  pub fn is_valid_name(name: &str) -> bool {
    is_var_name(name).is_ok()
  }

  #[inline]
  pub const fn as_str(&self) -> &str {
    match &self.name {
      Repr::Standard(name) => name.as_str(),
      Repr::Custom(boxed_name) => &boxed_name,
    }
  }
}

impl AsRef<str> for VarName {
  #[inline]
  fn as_ref(&self) -> &str {
    match &self.name {
      Repr::Standard(name) => name.as_str(),
      Repr::Custom(boxed_name) => &boxed_name,
    }
  }
}

impl std::fmt::Display for VarName {
  #[inline]
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self.name {
      Repr::Standard(name) => f.write_str(name.as_str()),
      Repr::Custom(boxed_name) => f.write_str(&boxed_name),
    }
  }
}

impl PartialEq<str> for VarName {
  #[inline]
  fn eq(&self, other: &str) -> bool {
    match &self.name {
      Repr::Standard(name) => name.as_str().eq(other),
      Repr::Custom(boxed_name) => boxed_name.as_ref().eq(other),
    }
  }
}

impl PartialEq<Box<str>> for VarName {
  #[inline]
  fn eq(&self, other: &Box<str>) -> bool {
    self.eq(other.as_ref())
  }
}

impl PartialEq<String> for VarName {
  #[inline]
  fn eq(&self, other: &String) -> bool {
    self.eq(other.as_str())
  }
}

impl PartialEq<VarName> for VarName {
  #[inline]
  fn eq(&self, other: &VarName) -> bool {
    match (&self.name, &other.name) {
      (Repr::Standard(lhs), Repr::Standard(rhs)) => lhs == rhs,
      _ => self.as_str().eq(other.as_str()),
    }
  }
}

impl PartialOrd for VarName {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.as_str().partial_cmp(other.as_str())
  }
}

impl Ord for VarName {
  #[inline]
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.as_str().cmp(other.as_str())
  }
}

#[cfg(feature = "serde")]
impl serde::Serialize for VarName {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    match &self.name {
      Repr::Standard(name) => serializer.serialize_str(name.as_str()),
      Repr::Custom(name) => serializer.serialize_str(&name),
    }
  }
}
