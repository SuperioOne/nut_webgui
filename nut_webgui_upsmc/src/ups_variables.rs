#![allow(dead_code)]

use super::errors::NutClientErrors;
use std::fmt::{Display, Formatter};

pub const VAR_AMBIENT_HUMIDITY: &str = "ambient.humidity";
pub const VAR_AMBIENT_HUMIDITY_ALARM: &str = "ambient.humidity.alarm";
pub const VAR_AMBIENT_HUMIDITY_ALARM_ENABLE: &str = "ambient.humidity.alarm.enable";
pub const VAR_AMBIENT_HUMIDITY_ALARM_MAXIMUM: &str = "ambient.humidity.alarm.maximum";
pub const VAR_AMBIENT_HUMIDITY_ALARM_MINIMUM: &str = "ambient.humidity.alarm.minimum";
pub const VAR_AMBIENT_HUMIDITY_HIGH: &str = "ambient.humidity.high";
pub const VAR_AMBIENT_HUMIDITY_HIGH_CRITICAL: &str = "ambient.humidity.high.critical";
pub const VAR_AMBIENT_HUMIDITY_HIGH_WARNING: &str = "ambient.humidity.high.warning";
pub const VAR_AMBIENT_HUMIDITY_LOW: &str = "ambient.humidity.low";
pub const VAR_AMBIENT_HUMIDITY_LOW_CRITICAL: &str = "ambient.humidity.low.critical";
pub const VAR_AMBIENT_HUMIDITY_LOW_WARNING: &str = "ambient.humidity.low.warning";
pub const VAR_AMBIENT_HUMIDITY_STATUS: &str = "ambient.humidity.status";
pub const VAR_AMBIENT_PRESENT: &str = "ambient.present";
pub const VAR_AMBIENT_TEMPERATURE: &str = "ambient.temperature";
pub const VAR_AMBIENT_TEMPERATURE_ALARM: &str = "ambient.temperature.alarm";
pub const VAR_AMBIENT_TEMPERATURE_ALARM_ENABLE: &str = "ambient.temperature.alarm.enable";
pub const VAR_AMBIENT_TEMPERATURE_ALARM_MAXIMUM: &str = "ambient.temperature.alarm.maximum";
pub const VAR_AMBIENT_TEMPERATURE_ALARM_MINIMUM: &str = "ambient.temperature.alarm.minimum";
pub const VAR_AMBIENT_TEMPERATURE_HIGH: &str = "ambient.temperature.high";
pub const VAR_AMBIENT_TEMPERATURE_HIGH_CRITICAL: &str = "ambient.temperature.high.critical";
pub const VAR_AMBIENT_TEMPERATURE_HIGH_WARNING: &str = "ambient.temperature.high.warning";
pub const VAR_AMBIENT_TEMPERATURE_LOW: &str = "ambient.temperature.low";
pub const VAR_AMBIENT_TEMPERATURE_LOW_CRITICAL: &str = "ambient.temperature.low.critical";
pub const VAR_AMBIENT_TEMPERATURE_LOW_WARNING: &str = "ambient.temperature.low.warning";
pub const VAR_AMBIENT_TEMPERATURE_STATUS: &str = "ambient.temperature.status";
pub const VAR_BATTERY_CAPACITY: &str = "battery.capacity";
pub const VAR_BATTERY_CHARGE: &str = "battery.charge";
pub const VAR_BATTERY_CHARGE_APPROX: &str = "battery.charge.approx";
pub const VAR_BATTERY_CHARGE_LOW: &str = "battery.charge.low";
pub const VAR_BATTERY_CHARGE_RESTART: &str = "battery.charge.restart";
pub const VAR_BATTERY_CHARGE_WARNING: &str = "battery.charge.warning";
pub const VAR_BATTERY_CURRENT: &str = "battery.current";
pub const VAR_BATTERY_ENERGYSAVE: &str = "battery.energysave";
pub const VAR_BATTERY_ENERGYSAVE_DELAY: &str = "battery.energysave.delay";
pub const VAR_BATTERY_ENERGYSAVE_LOAD: &str = "battery.energysave.load";
pub const VAR_BATTERY_ENERGYSAVE_REALPOWER: &str = "battery.energysave.realpower";
pub const VAR_BATTERY_PROTECTION: &str = "battery.protection";
pub const VAR_BATTERY_RUNTIME: &str = "battery.runtime";
pub const VAR_BATTERY_RUNTIME_LOW: &str = "battery.runtime.low";
pub const VAR_BATTERY_TEMPERATURE: &str = "battery.temperature";
pub const VAR_BATTERY_TYPE: &str = "battery.type";
pub const VAR_BATTERY_VOLTAGE: &str = "battery.voltage";
pub const VAR_BATTERY_VOLTAGE_NOMINAL: &str = "battery.voltage.nominal";
pub const VAR_DEVICE_MFR: &str = "device.mfr";
pub const VAR_DEVICE_MODEL: &str = "device.model";
pub const VAR_DEVICE_SERIAL: &str = "device.serial";
pub const VAR_DEVICE_TYPE: &str = "device.type";
pub const VAR_DRIVER_NAME: &str = "driver.name";
pub const VAR_DRIVER_PARAMETER_LOWBATT: &str = "driver.parameter.lowbatt";
pub const VAR_DRIVER_PARAMETER_OFFDELAY: &str = "driver.parameter.offdelay";
pub const VAR_DRIVER_PARAMETER_ONDELAY: &str = "driver.parameter.ondelay";
pub const VAR_DRIVER_PARAMETER_POLLFREQ: &str = "driver.parameter.pollfreq";
pub const VAR_DRIVER_PARAMETER_POLLINTERVAL: &str = "driver.parameter.pollinterval";
pub const VAR_DRIVER_PARAMETER_PORT: &str = "driver.parameter.port";
pub const VAR_DRIVER_PARAMETER_SYNCHRONOUS: &str = "driver.parameter.synchronous";
pub const VAR_DRIVER_PARAMETER_VENDORID: &str = "driver.parameter.vendorid";
pub const VAR_DRIVER_VERSION: &str = "driver.version";
pub const VAR_DRIVER_VERSION_DATA: &str = "driver.version.data";
pub const VAR_DRIVER_VERSION_INTERNAL: &str = "driver.version.internal";
pub const VAR_INPUT_CURRENT: &str = "input.current";
pub const VAR_INPUT_CURRENT_NOMINAL: &str = "input.current.nominal";
pub const VAR_INPUT_CURRENT_STATUS: &str = "input.current.status";
pub const VAR_INPUT_FREQUENCY: &str = "input.frequency";
pub const VAR_INPUT_FREQUENCY_EXTENDED: &str = "input.frequency.extended";
pub const VAR_INPUT_FREQUENCY_HIGH: &str = "input.frequency.high";
pub const VAR_INPUT_FREQUENCY_LOW: &str = "input.frequency.low";
pub const VAR_INPUT_FREQUENCY_NOMINAL: &str = "input.frequency.nominal";
pub const VAR_INPUT_FREQUENCY_STATUS: &str = "input.frequency.status";
pub const VAR_INPUT_LOAD: &str = "input.load";
pub const VAR_INPUT_POWER: &str = "input.power";
pub const VAR_INPUT_QUALITY: &str = "input.quality";
pub const VAR_INPUT_REALPOWER: &str = "input.realpower";
pub const VAR_INPUT_SENSITIVITY: &str = "input.sensitivity";
pub const VAR_INPUT_SOURCE: &str = "input.source";
pub const VAR_INPUT_SOURCE_PREFERRED: &str = "input.source.preferred";
pub const VAR_INPUT_TRANSFER_BOOST_HIGH: &str = "input.transfer.boost.high";
pub const VAR_INPUT_TRANSFER_BOOST_LOW: &str = "input.transfer.boost.low";
pub const VAR_INPUT_TRANSFER_DELAY: &str = "input.transfer.delay";
pub const VAR_INPUT_TRANSFER_HIGH: &str = "input.transfer.high";
pub const VAR_INPUT_TRANSFER_LOW: &str = "input.transfer.low";
pub const VAR_INPUT_TRANSFER_TRIM_HIGH: &str = "input.transfer.trim.high";
pub const VAR_INPUT_TRANSFER_TRIM_LOW: &str = "input.transfer.trim.low";
pub const VAR_INPUT_VOLTAGE: &str = "input.voltage";
pub const VAR_INPUT_VOLTAGE_NOMINAL: &str = "input.voltage.nominal";
pub const VAR_OUTPUT_CURRENT: &str = "output.current";
pub const VAR_OUTPUT_CURRENT_NOMINAL: &str = "output.current.nominal";
pub const VAR_OUTPUT_FREQUENCY: &str = "output.frequency";
pub const VAR_OUTPUT_FREQUENCY_NOMINAL: &str = "output.frequency.nominal";
pub const VAR_OUTPUT_VOLTAGE: &str = "output.voltage";
pub const VAR_OUTPUT_VOLTAGE_NOMINAL: &str = "output.voltage.nominal";
pub const VAR_UPS_BEEPER_STATUS: &str = "ups.beeper.status";
pub const VAR_UPS_CONTACTS: &str = "ups.contacts";
pub const VAR_UPS_DELAY_SHUTDOWN: &str = "ups.delay.shutdown";
pub const VAR_UPS_DELAY_START: &str = "ups.delay.start";
pub const VAR_UPS_FIRMWARE: &str = "ups.firmware";
pub const VAR_UPS_LOAD: &str = "ups.load";
pub const VAR_UPS_MFR: &str = "ups.mfr";
pub const VAR_UPS_MODEL: &str = "ups.model";
pub const VAR_UPS_POWER: &str = "ups.power";
pub const VAR_UPS_POWER_NOMINAL: &str = "ups.power.nominal";
pub const VAR_UPS_PRODUCTID: &str = "ups.productid";
pub const VAR_UPS_REALPOWER: &str = "ups.realpower";
pub const VAR_UPS_REALPOWER_NOMINAL: &str = "ups.realpower.nominal";
pub const VAR_UPS_SERIAL: &str = "ups.serial";
pub const VAR_UPS_STATUS: &str = "ups.status";
pub const VAR_UPS_TEMPERATURE: &str = "ups.temperature";
pub const VAR_UPS_TEST_INTERVAL: &str = "ups.test.interval";
pub const VAR_UPS_TEST_RESULT: &str = "ups.test.result";
pub const VAR_UPS_TIMER_SHUTDOWN: &str = "ups.timer.shutdown";
pub const VAR_UPS_TIMER_START: &str = "ups.timer.start";
pub const VAR_UPS_VENDORID: &str = "ups.vendorid";

// /raw.githubusercontent.com/networkupstools/nut/7b225f5291da7fb98003932ffda4e99deb7f23d3/data/cmdvartab

pub const STATUS_ALARM: &str = "ALARM";
pub const STATUS_BOOST: &str = "BOOST";
pub const STATUS_BYPASS: &str = "BYPASS";
pub const STATUS_CAL: &str = "CAL";
pub const STATUS_CHRG: &str = "CHRG";
pub const STATUS_COMM: &str = "COMM";
pub const STATUS_DISCHRG: &str = "DISCHRG";
pub const STATUS_FSD: &str = "FSD";
pub const STATUS_LB: &str = "LB";
pub const STATUS_NOCOMM: &str = "NOCOMM";
pub const STATUS_OB: &str = "OB";
pub const STATUS_OFF: &str = "OFF";
pub const STATUS_OL: &str = "OL";
pub const STATUS_OVER: &str = "OVER";
pub const STATUS_RB: &str = "RB";
pub const STATUS_TEST: &str = "TEST";
pub const STATUS_TICK: &str = "TICK";
pub const STATUS_TOCK: &str = "TOCK";
pub const STATUS_TRIM: &str = "TRIM";

pub const ERR_ACCESS_DENIED: &str = "ACCESS-DENIED";
pub const ERR_ALREADY_ATTACHED: &str = "ALREADY-ATTACHED";
pub const ERR_ALREADY_SET_PASSWORD: &str = "ALREADY-SET-PASSWORD";
pub const ERR_ALREADY_SET_USERNAME: &str = "ALREADY-SET-USERNAME";
pub const ERR_CMD_NOT_SUPPORTED: &str = "CMD-NOT-SUPPORTED";
pub const ERR_DATE_STALE: &str = "DATA-STALE";
pub const ERR_DRIVER_NOT_CONNECTED: &str = "DRIVER-NOT-CONNECTED";
pub const ERR_FEATURE_NOT_CONFIGURED: &str = "FEATURE-NOT-CONFIGURED";
pub const ERR_FEATURE_NOT_SUPPORTED: &str = "FEATURE-NOT-SUPPORTED";
pub const ERR_INSTCMD_FAILED: &str = "INSTCMD-FAILED";
pub const ERR_INVALID_ARGUMENT: &str = "INVALID-ARGUMENT";
pub const ERR_INVALID_PASSWORD: &str = "INVALID-PASSWORD";
pub const ERR_INVALID_USERNAME: &str = "INVALID-USERNAME";
pub const ERR_INVALID_VALUE: &str = "INVALID-VALUE";
pub const ERR_PASSWORD_REQUIRED: &str = "PASSWORD-REQUIRED";
pub const ERR_READONLY: &str = "READONLY";
pub const ERR_SET_FAILED: &str = "SET-FAILED";
pub const ERR_TLS_ALREADY_ENABLED: &str = "TLS-ALREADY-ENABLED";
pub const ERR_TLS_NOT_ENABLED: &str = "TLS-NOT-ENABLED";
pub const ERR_TOO_LONG: &str = "TOO-LONG";
pub const ERR_UNKNOWN_COMMAND: &str = "UNKNOWN-COMMAND";
pub const ERR_UNKNOWN_UPS: &str = "UNKNOWN-UPS";
pub const ERR_USERNAME_REQUIRED: &str = "USERNAME-REQUIRED";
pub const ERR_VAR_NOT_SUPPORTED: &str = "VAR-NOT-SUPPORTED";

#[derive(Debug, Clone, PartialEq)]
pub enum UpsVariable {
  BatteryCharge(f64),
  BatteryChargeLow(f64),
  BatteryRuntime(f64),
  BatteryTemperature(f64),
  BatteryType(String),
  BatteryVoltage(f64),
  BatteryVoltageNominal(f64),
  DeviceMfr(String),
  DeviceModel(String),
  DeviceSerial(String),
  DeviceType(String),
  DriverName(String),
  DriverParameterLowBatt(i32),
  DriverParameterOffDelay(i32),
  DriverParameterOnDelay(i32),
  DriverParameterPollFreq(i32),
  DriverParameterPollInterval(i32),
  DriverParameterPort(String),
  DriverParameterSynchronous(String),
  DriverParameterVendorId(String),
  DriverVersion(String),
  DriverVersionData(String),
  DriverVersionInternal(String),
  Generic(String, String),
  InputTransferHigh(f64),
  InputTransferLow(f64),
  InputVoltage(f64),
  InputVoltageNominal(f64),
  OutputFrequencyNominal(String),
  OutputVoltage(String),
  OutputVoltageNominal(String),
  UpsBeeperStatus(String),
  UpsDelayShutdown(i32),
  UpsDelayStart(i32),
  UpsFirmware(String),
  UpsLoad(f64),
  UpsMfr(String),
  UpsModel(String),
  UpsPower(f64),
  UpsPowerNominal(f64),
  UpsProductId(String),
  UpsRealPower(f64),
  UpsRealPowerNominal(f64),
  UpsSerial(String),
  UpsStatus(UpsStatus),
  UpsTemperature(f64),
  UpsTimerShutdown(f64),
  UpsTimerStart(f64),
  UpsVendorId(String),
}

#[cfg(feature = "serde")]
impl serde::Serialize for UpsVariable {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    match self {
      UpsVariable::BatteryCharge(value) => serializer.serialize_f64(*value),
      UpsVariable::BatteryChargeLow(value) => serializer.serialize_f64(*value),
      UpsVariable::BatteryRuntime(value) => serializer.serialize_f64(*value),
      UpsVariable::BatteryTemperature(value) => serializer.serialize_f64(*value),
      UpsVariable::BatteryType(value) => serializer.serialize_str(value),
      UpsVariable::BatteryVoltage(value) => serializer.serialize_f64(*value),
      UpsVariable::BatteryVoltageNominal(value) => serializer.serialize_f64(*value),
      UpsVariable::DeviceMfr(value) => serializer.serialize_str(value),
      UpsVariable::DeviceModel(value) => serializer.serialize_str(value),
      UpsVariable::DeviceSerial(value) => serializer.serialize_str(value),
      UpsVariable::DeviceType(value) => serializer.serialize_str(value),
      UpsVariable::DriverName(value) => serializer.serialize_str(value),
      UpsVariable::DriverParameterLowBatt(value) => serializer.serialize_i32(*value),
      UpsVariable::DriverParameterOffDelay(value) => serializer.serialize_i32(*value),
      UpsVariable::DriverParameterOnDelay(value) => serializer.serialize_i32(*value),
      UpsVariable::DriverParameterPollFreq(value) => serializer.serialize_i32(*value),
      UpsVariable::DriverParameterPollInterval(value) => serializer.serialize_i32(*value),
      UpsVariable::DriverParameterPort(value) => serializer.serialize_str(value),
      UpsVariable::DriverParameterSynchronous(value) => serializer.serialize_str(value),
      UpsVariable::DriverParameterVendorId(value) => serializer.serialize_str(value),
      UpsVariable::DriverVersion(value) => serializer.serialize_str(value),
      UpsVariable::DriverVersionData(value) => serializer.serialize_str(value),
      UpsVariable::DriverVersionInternal(value) => serializer.serialize_str(value),
      UpsVariable::Generic(_, value) => serializer.serialize_str(value),
      UpsVariable::InputTransferHigh(value) => serializer.serialize_f64(*value),
      UpsVariable::InputTransferLow(value) => serializer.serialize_f64(*value),
      UpsVariable::InputVoltage(value) => serializer.serialize_f64(*value),
      UpsVariable::InputVoltageNominal(value) => serializer.serialize_f64(*value),
      UpsVariable::OutputFrequencyNominal(value) => serializer.serialize_str(value),
      UpsVariable::OutputVoltage(value) => serializer.serialize_str(value),
      UpsVariable::OutputVoltageNominal(value) => serializer.serialize_str(value),
      UpsVariable::UpsBeeperStatus(value) => serializer.serialize_str(value),
      UpsVariable::UpsDelayShutdown(value) => serializer.serialize_i32(*value),
      UpsVariable::UpsDelayStart(value) => serializer.serialize_i32(*value),
      UpsVariable::UpsFirmware(value) => serializer.serialize_str(value),
      UpsVariable::UpsLoad(value) => serializer.serialize_f64(*value),
      UpsVariable::UpsMfr(value) => serializer.serialize_str(value),
      UpsVariable::UpsModel(value) => serializer.serialize_str(value),
      UpsVariable::UpsPower(value) => serializer.serialize_f64(*value),
      UpsVariable::UpsPowerNominal(value) => serializer.serialize_f64(*value),
      UpsVariable::UpsProductId(value) => serializer.serialize_str(value),
      UpsVariable::UpsRealPower(value) => serializer.serialize_f64(*value),
      UpsVariable::UpsRealPowerNominal(value) => serializer.serialize_f64(*value),
      UpsVariable::UpsSerial(value) => serializer.serialize_str(value),
      UpsVariable::UpsStatus(value) => serializer.serialize_str(value.as_str()),
      UpsVariable::UpsTemperature(value) => serializer.serialize_f64(*value),
      UpsVariable::UpsTimerShutdown(value) => serializer.serialize_f64(*value),
      UpsVariable::UpsTimerStart(value) => serializer.serialize_f64(*value),
      UpsVariable::UpsVendorId(value) => serializer.serialize_str(value),
    }
  }
}

impl UpsVariable {
  pub fn name(&self) -> &str {
    match self {
      UpsVariable::BatteryCharge(_) => VAR_BATTERY_CHARGE,
      UpsVariable::BatteryChargeLow(_) => VAR_BATTERY_CHARGE_LOW,
      UpsVariable::BatteryRuntime(_) => VAR_BATTERY_RUNTIME,
      UpsVariable::BatteryTemperature(_) => VAR_BATTERY_TEMPERATURE,
      UpsVariable::BatteryType(_) => VAR_BATTERY_TYPE,
      UpsVariable::BatteryVoltage(_) => VAR_BATTERY_VOLTAGE,
      UpsVariable::BatteryVoltageNominal(_) => VAR_BATTERY_VOLTAGE_NOMINAL,
      UpsVariable::DeviceMfr(_) => VAR_DEVICE_MFR,
      UpsVariable::DeviceModel(_) => VAR_DEVICE_MODEL,
      UpsVariable::DeviceSerial(_) => VAR_DEVICE_SERIAL,
      UpsVariable::DeviceType(_) => VAR_DEVICE_TYPE,
      UpsVariable::DriverName(_) => VAR_DRIVER_NAME,
      UpsVariable::DriverParameterLowBatt(_) => VAR_DRIVER_PARAMETER_LOWBATT,
      UpsVariable::DriverParameterOffDelay(_) => VAR_DRIVER_PARAMETER_OFFDELAY,
      UpsVariable::DriverParameterOnDelay(_) => VAR_DRIVER_PARAMETER_ONDELAY,
      UpsVariable::DriverParameterPollFreq(_) => VAR_DRIVER_PARAMETER_POLLFREQ,
      UpsVariable::DriverParameterPollInterval(_) => VAR_DRIVER_PARAMETER_POLLINTERVAL,
      UpsVariable::DriverParameterPort(_) => VAR_DRIVER_PARAMETER_PORT,
      UpsVariable::DriverParameterSynchronous(_) => VAR_DRIVER_PARAMETER_SYNCHRONOUS,
      UpsVariable::DriverParameterVendorId(_) => VAR_DRIVER_PARAMETER_VENDORID,
      UpsVariable::DriverVersion(_) => VAR_DRIVER_VERSION,
      UpsVariable::DriverVersionData(_) => VAR_DRIVER_VERSION_DATA,
      UpsVariable::DriverVersionInternal(_) => VAR_DRIVER_VERSION_INTERNAL,
      UpsVariable::Generic(name, _) => name,
      UpsVariable::InputTransferHigh(_) => VAR_INPUT_TRANSFER_HIGH,
      UpsVariable::InputTransferLow(_) => VAR_INPUT_TRANSFER_LOW,
      UpsVariable::InputVoltage(_) => VAR_INPUT_VOLTAGE,
      UpsVariable::InputVoltageNominal(_) => VAR_INPUT_VOLTAGE_NOMINAL,
      UpsVariable::OutputFrequencyNominal(_) => VAR_OUTPUT_FREQUENCY_NOMINAL,
      UpsVariable::OutputVoltage(_) => VAR_OUTPUT_VOLTAGE,
      UpsVariable::OutputVoltageNominal(_) => VAR_OUTPUT_VOLTAGE_NOMINAL,
      UpsVariable::UpsBeeperStatus(_) => VAR_UPS_BEEPER_STATUS,
      UpsVariable::UpsDelayShutdown(_) => VAR_UPS_DELAY_SHUTDOWN,
      UpsVariable::UpsDelayStart(_) => VAR_UPS_DELAY_START,
      UpsVariable::UpsFirmware(_) => VAR_UPS_FIRMWARE,
      UpsVariable::UpsLoad(_) => VAR_UPS_LOAD,
      UpsVariable::UpsMfr(_) => VAR_UPS_MFR,
      UpsVariable::UpsModel(_) => VAR_UPS_MODEL,
      UpsVariable::UpsPower(_) => VAR_UPS_POWER,
      UpsVariable::UpsPowerNominal(_) => VAR_UPS_POWER_NOMINAL,
      UpsVariable::UpsProductId(_) => VAR_UPS_PRODUCTID,
      UpsVariable::UpsRealPower(_) => VAR_UPS_REALPOWER,
      UpsVariable::UpsRealPowerNominal(_) => VAR_UPS_REALPOWER_NOMINAL,
      UpsVariable::UpsSerial(_) => VAR_UPS_SERIAL,
      UpsVariable::UpsStatus(_) => VAR_UPS_STATUS,
      UpsVariable::UpsTemperature(_) => VAR_UPS_TEMPERATURE,
      UpsVariable::UpsTimerShutdown(_) => VAR_UPS_TIMER_SHUTDOWN,
      UpsVariable::UpsTimerStart(_) => VAR_UPS_TIMER_START,
      UpsVariable::UpsVendorId(_) => VAR_UPS_VENDORID,
    }
  }

  pub fn value_as_string(&self) -> String {
    match self {
      UpsVariable::BatteryCharge(val) => val.to_string(),
      UpsVariable::BatteryChargeLow(val) => val.to_string(),
      UpsVariable::BatteryRuntime(val) => val.to_string(),
      UpsVariable::BatteryTemperature(val) => val.to_string(),
      UpsVariable::BatteryType(val) => val.to_string(),
      UpsVariable::BatteryVoltage(val) => val.to_string(),
      UpsVariable::BatteryVoltageNominal(val) => val.to_string(),
      UpsVariable::DeviceMfr(val) => val.to_string(),
      UpsVariable::DeviceModel(val) => val.to_string(),
      UpsVariable::DeviceSerial(val) => val.to_string(),
      UpsVariable::DeviceType(val) => val.to_string(),
      UpsVariable::DriverName(val) => val.to_string(),
      UpsVariable::DriverParameterLowBatt(val) => val.to_string(),
      UpsVariable::DriverParameterOffDelay(val) => val.to_string(),
      UpsVariable::DriverParameterOnDelay(val) => val.to_string(),
      UpsVariable::DriverParameterPollFreq(val) => val.to_string(),
      UpsVariable::DriverParameterPollInterval(val) => val.to_string(),
      UpsVariable::DriverParameterPort(val) => val.to_string(),
      UpsVariable::DriverParameterSynchronous(val) => val.to_string(),
      UpsVariable::DriverParameterVendorId(val) => val.to_string(),
      UpsVariable::DriverVersion(val) => val.to_string(),
      UpsVariable::DriverVersionData(val) => val.to_string(),
      UpsVariable::DriverVersionInternal(val) => val.to_string(),
      UpsVariable::Generic(_, val) => val.to_string(),
      UpsVariable::InputTransferHigh(val) => val.to_string(),
      UpsVariable::InputTransferLow(val) => val.to_string(),
      UpsVariable::InputVoltage(val) => val.to_string(),
      UpsVariable::InputVoltageNominal(val) => val.to_string(),
      UpsVariable::OutputFrequencyNominal(val) => val.to_string(),
      UpsVariable::OutputVoltage(val) => val.to_string(),
      UpsVariable::OutputVoltageNominal(val) => val.to_string(),
      UpsVariable::UpsBeeperStatus(val) => val.to_string(),
      UpsVariable::UpsDelayShutdown(val) => val.to_string(),
      UpsVariable::UpsDelayStart(val) => val.to_string(),
      UpsVariable::UpsFirmware(val) => val.to_string(),
      UpsVariable::UpsLoad(val) => val.to_string(),
      UpsVariable::UpsMfr(val) => val.to_string(),
      UpsVariable::UpsModel(val) => val.to_string(),
      UpsVariable::UpsPower(val) => val.to_string(),
      UpsVariable::UpsPowerNominal(val) => val.to_string(),
      UpsVariable::UpsProductId(val) => val.to_string(),
      UpsVariable::UpsRealPower(val) => val.to_string(),
      UpsVariable::UpsRealPowerNominal(val) => val.to_string(),
      UpsVariable::UpsSerial(val) => val.to_string(),
      UpsVariable::UpsStatus(val) => val.to_string(),
      UpsVariable::UpsTemperature(val) => val.to_string(),
      UpsVariable::UpsTimerShutdown(val) => val.to_string(),
      UpsVariable::UpsTimerStart(val) => val.to_string(),
      UpsVariable::UpsVendorId(val) => val.to_string(),
    }
  }
}

impl TryFrom<(&str, &str)> for UpsVariable {
  type Error = NutClientErrors;
  fn try_from(from_value: (&str, &str)) -> Result<Self, Self::Error> {
    let (name, value) = from_value;
    let ups_variable = match name {
      VAR_BATTERY_CHARGE => UpsVariable::BatteryCharge(value.parse::<f64>()?),
      VAR_BATTERY_CHARGE_LOW => UpsVariable::BatteryChargeLow(value.parse::<f64>()?),
      VAR_BATTERY_RUNTIME => UpsVariable::BatteryRuntime(value.parse::<f64>()?),
      VAR_BATTERY_TEMPERATURE => UpsVariable::BatteryTemperature(value.parse::<f64>()?),
      VAR_BATTERY_TYPE => UpsVariable::BatteryType(value.into()),
      VAR_BATTERY_VOLTAGE => UpsVariable::BatteryVoltage(value.parse::<f64>()?),
      VAR_BATTERY_VOLTAGE_NOMINAL => UpsVariable::BatteryVoltageNominal(value.parse::<f64>()?),
      VAR_DEVICE_MFR => UpsVariable::DeviceMfr(value.into()),
      VAR_DEVICE_MODEL => UpsVariable::DeviceModel(value.into()),
      VAR_DEVICE_SERIAL => UpsVariable::DeviceSerial(value.into()),
      VAR_DEVICE_TYPE => UpsVariable::DeviceType(value.into()),
      VAR_DRIVER_NAME => UpsVariable::DriverName(value.into()),
      VAR_DRIVER_PARAMETER_LOWBATT => UpsVariable::DriverParameterLowBatt(value.parse::<i32>()?),
      VAR_DRIVER_PARAMETER_OFFDELAY => UpsVariable::DriverParameterOffDelay(value.parse::<i32>()?),
      VAR_DRIVER_PARAMETER_ONDELAY => UpsVariable::DriverParameterOnDelay(value.parse::<i32>()?),
      VAR_DRIVER_PARAMETER_POLLFREQ => UpsVariable::DriverParameterPollFreq(value.parse::<i32>()?),
      VAR_DRIVER_PARAMETER_POLLINTERVAL => {
        UpsVariable::DriverParameterPollInterval(value.parse::<i32>()?)
      }
      VAR_DRIVER_PARAMETER_PORT => UpsVariable::DriverParameterPort(value.into()),
      VAR_DRIVER_PARAMETER_SYNCHRONOUS => UpsVariable::DriverParameterSynchronous(value.into()),
      VAR_DRIVER_PARAMETER_VENDORID => UpsVariable::DriverParameterVendorId(value.into()),
      VAR_DRIVER_VERSION => UpsVariable::DriverVersion(value.into()),
      VAR_DRIVER_VERSION_DATA => UpsVariable::DriverVersionData(value.into()),
      VAR_DRIVER_VERSION_INTERNAL => UpsVariable::DriverVersionInternal(value.into()),
      VAR_INPUT_TRANSFER_HIGH => UpsVariable::InputTransferHigh(value.parse::<f64>()?),
      VAR_INPUT_TRANSFER_LOW => UpsVariable::InputTransferLow(value.parse::<f64>()?),
      VAR_INPUT_VOLTAGE => UpsVariable::InputVoltage(value.parse::<f64>()?),
      VAR_INPUT_VOLTAGE_NOMINAL => UpsVariable::InputVoltageNominal(value.parse::<f64>()?),
      VAR_OUTPUT_FREQUENCY_NOMINAL => UpsVariable::OutputFrequencyNominal(value.into()),
      VAR_OUTPUT_VOLTAGE => UpsVariable::OutputVoltage(value.into()),
      VAR_OUTPUT_VOLTAGE_NOMINAL => UpsVariable::OutputVoltageNominal(value.into()),
      VAR_UPS_BEEPER_STATUS => UpsVariable::UpsBeeperStatus(value.into()),
      VAR_UPS_DELAY_SHUTDOWN => UpsVariable::UpsDelayShutdown(value.parse::<i32>()?),
      VAR_UPS_DELAY_START => UpsVariable::UpsDelayStart(value.parse::<i32>()?),
      VAR_UPS_FIRMWARE => UpsVariable::UpsFirmware(value.into()),
      VAR_UPS_LOAD => UpsVariable::UpsLoad(value.parse::<f64>()?),
      VAR_UPS_MFR => UpsVariable::UpsMfr(value.into()),
      VAR_UPS_MODEL => UpsVariable::UpsModel(value.into()),
      VAR_UPS_POWER => UpsVariable::UpsPower(value.parse::<f64>()?),
      VAR_UPS_POWER_NOMINAL => UpsVariable::UpsPowerNominal(value.parse::<f64>()?),
      VAR_UPS_REALPOWER => UpsVariable::UpsRealPower(value.parse::<f64>()?),
      VAR_UPS_REALPOWER_NOMINAL => UpsVariable::UpsRealPowerNominal(value.parse::<f64>()?),
      VAR_UPS_PRODUCTID => UpsVariable::UpsProductId(value.into()),
      VAR_UPS_SERIAL => UpsVariable::UpsSerial(value.into()),
      VAR_UPS_STATUS => UpsVariable::UpsStatus(value.into()),
      VAR_UPS_TEMPERATURE => UpsVariable::UpsTemperature(value.parse::<f64>()?),
      VAR_UPS_TIMER_SHUTDOWN => UpsVariable::UpsTimerShutdown(value.parse::<f64>()?),
      VAR_UPS_TIMER_START => UpsVariable::UpsTimerStart(value.parse::<f64>()?),
      VAR_UPS_VENDORID => UpsVariable::UpsVendorId(value.into()),
      param => UpsVariable::Generic(param.to_owned(), value.to_owned()),
    };

    Ok(ups_variable)
  }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum UpsStatus {
  Alarm,
  Boost,
  Bypass,
  Calibrating,
  Charging,
  COMM,
  Discharging,
  ForcedShutdown,
  LowBattery,
  NoCOMM,
  OnBattery,
  Offline,
  Online,
  Overloaded,
  ReplaceBattery,
  Test,
  Tick,
  Tock,
  Trim,
  Unknown(String),
}

impl UpsStatus {
  pub fn as_str(&self) -> &str {
    match self {
      UpsStatus::Alarm => STATUS_ALARM,
      UpsStatus::Boost => STATUS_BOOST,
      UpsStatus::Bypass => STATUS_BYPASS,
      UpsStatus::Calibrating => STATUS_CAL,
      UpsStatus::Charging => STATUS_CHRG,
      UpsStatus::COMM => STATUS_COMM,
      UpsStatus::Discharging => STATUS_DISCHRG,
      UpsStatus::ForcedShutdown => STATUS_FSD,
      UpsStatus::LowBattery => STATUS_LB,
      UpsStatus::NoCOMM => STATUS_NOCOMM,
      UpsStatus::OnBattery => STATUS_OB,
      UpsStatus::Offline => STATUS_OFF,
      UpsStatus::Online => STATUS_OL,
      UpsStatus::Overloaded => STATUS_OVER,
      UpsStatus::ReplaceBattery => STATUS_RB,
      UpsStatus::Test => STATUS_TEST,
      UpsStatus::Tick => STATUS_TICK,
      UpsStatus::Tock => STATUS_TOCK,
      UpsStatus::Trim => STATUS_TRIM,
      UpsStatus::Unknown(value) => value,
    }
  }
}

impl<T> From<T> for UpsStatus
where
  T: AsRef<str>,
{
  fn from(value: T) -> Self {
    match value.as_ref() {
      STATUS_ALARM => UpsStatus::Alarm,
      STATUS_BOOST => UpsStatus::Boost,
      STATUS_BYPASS => UpsStatus::Bypass,
      STATUS_CAL => UpsStatus::Calibrating,
      STATUS_CHRG => UpsStatus::Charging,
      STATUS_COMM => UpsStatus::COMM,
      STATUS_DISCHRG => UpsStatus::Discharging,
      STATUS_FSD => UpsStatus::ForcedShutdown,
      STATUS_LB => UpsStatus::LowBattery,
      STATUS_NOCOMM => UpsStatus::NoCOMM,
      STATUS_OB => UpsStatus::OnBattery,
      STATUS_OFF => UpsStatus::Offline,
      STATUS_OL => UpsStatus::Online,
      STATUS_OVER => UpsStatus::Overloaded,
      STATUS_RB => UpsStatus::ReplaceBattery,
      STATUS_TEST => UpsStatus::Test,
      STATUS_TICK => UpsStatus::Tick,
      STATUS_TOCK => UpsStatus::Tock,
      STATUS_TRIM => UpsStatus::Trim,
      unknown => UpsStatus::Unknown(unknown.to_owned()),
    }
  }
}

impl Display for UpsStatus {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.as_str())
  }
}

#[derive(Debug, Eq, PartialEq)]
pub enum UpsError {
  AccessDenied,
  AlreadyAttached,
  AlreadySetPassword,
  AlreadySetUsername,
  CmdNotSupported,
  DataStale,
  DriverNotConnected,
  FeatureNotConfigured,
  FeatureNotSupported,
  InstCmdFailed,
  InvalidArgument,
  InvalidPassword,
  InvalidUsername,
  InvalidValue,
  PasswordRequired,
  READONLY,
  SetFailed,
  TlsAlreadyEnabled,
  TlsNotEnabled,
  TooLong,
  UnknownCommand,
  UnknownUps,
  UsernameRequired,
  VarNotSupported,
  Unknown(String),
}

impl Display for UpsError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let error_text = match self {
      UpsError::AccessDenied => ERR_ACCESS_DENIED,
      UpsError::AlreadyAttached => ERR_ALREADY_ATTACHED,
      UpsError::AlreadySetPassword => ERR_ALREADY_SET_PASSWORD,
      UpsError::AlreadySetUsername => ERR_ALREADY_SET_USERNAME,
      UpsError::CmdNotSupported => ERR_CMD_NOT_SUPPORTED,
      UpsError::DataStale => ERR_DATE_STALE,
      UpsError::DriverNotConnected => ERR_DRIVER_NOT_CONNECTED,
      UpsError::FeatureNotConfigured => ERR_FEATURE_NOT_CONFIGURED,
      UpsError::FeatureNotSupported => ERR_FEATURE_NOT_SUPPORTED,
      UpsError::InstCmdFailed => ERR_INSTCMD_FAILED,
      UpsError::InvalidArgument => ERR_INVALID_ARGUMENT,
      UpsError::InvalidPassword => ERR_INVALID_PASSWORD,
      UpsError::InvalidUsername => ERR_INVALID_USERNAME,
      UpsError::InvalidValue => ERR_INVALID_VALUE,
      UpsError::PasswordRequired => ERR_PASSWORD_REQUIRED,
      UpsError::READONLY => ERR_READONLY,
      UpsError::SetFailed => ERR_SET_FAILED,
      UpsError::TlsAlreadyEnabled => ERR_TLS_ALREADY_ENABLED,
      UpsError::TlsNotEnabled => ERR_TLS_NOT_ENABLED,
      UpsError::TooLong => ERR_TOO_LONG,
      UpsError::UnknownCommand => ERR_UNKNOWN_COMMAND,
      UpsError::UnknownUps => ERR_UNKNOWN_UPS,
      UpsError::UsernameRequired => ERR_USERNAME_REQUIRED,
      UpsError::VarNotSupported => ERR_VAR_NOT_SUPPORTED,
      UpsError::Unknown(value) => value.as_str(),
    };

    f.write_str(error_text)
  }
}

impl<T> From<T> for UpsError
where
  T: AsRef<str>,
{
  fn from(value: T) -> Self {
    match value.as_ref() {
      ERR_ACCESS_DENIED => UpsError::AccessDenied,
      ERR_ALREADY_ATTACHED => UpsError::AlreadyAttached,
      ERR_ALREADY_SET_PASSWORD => UpsError::AlreadySetPassword,
      ERR_ALREADY_SET_USERNAME => UpsError::AlreadySetUsername,
      ERR_CMD_NOT_SUPPORTED => UpsError::CmdNotSupported,
      ERR_DATE_STALE => UpsError::DataStale,
      ERR_DRIVER_NOT_CONNECTED => UpsError::DriverNotConnected,
      ERR_FEATURE_NOT_CONFIGURED => UpsError::FeatureNotConfigured,
      ERR_FEATURE_NOT_SUPPORTED => UpsError::FeatureNotSupported,
      ERR_INSTCMD_FAILED => UpsError::InstCmdFailed,
      ERR_INVALID_ARGUMENT => UpsError::InvalidArgument,
      ERR_INVALID_PASSWORD => UpsError::InvalidPassword,
      ERR_INVALID_USERNAME => UpsError::InvalidUsername,
      ERR_INVALID_VALUE => UpsError::InvalidValue,
      ERR_PASSWORD_REQUIRED => UpsError::PasswordRequired,
      ERR_READONLY => UpsError::READONLY,
      ERR_SET_FAILED => UpsError::SetFailed,
      ERR_TLS_ALREADY_ENABLED => UpsError::TlsAlreadyEnabled,
      ERR_TLS_NOT_ENABLED => UpsError::TlsNotEnabled,
      ERR_TOO_LONG => UpsError::TooLong,
      ERR_UNKNOWN_COMMAND => UpsError::UnknownCommand,
      ERR_UNKNOWN_UPS => UpsError::UnknownUps,
      ERR_USERNAME_REQUIRED => UpsError::UsernameRequired,
      ERR_VAR_NOT_SUPPORTED => UpsError::VarNotSupported,
      unknown => UpsError::Unknown(unknown.to_owned()),
    }
  }
}
