use super::errors::NutClientErrors;
use serde::{Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::num::{ParseFloatError, ParseIntError};

// https://raw.githubusercontent.com/networkupstools/nut/7b225f5291da7fb98003932ffda4e99deb7f23d3/data/cmdvartab

pub(crate) const VAR_BATTERY_CHARGE: &'static str = "battery.charge";
pub(crate) const VAR_BATTERY_CHARGE_LOW: &'static str = "battery.charge.low";
pub(crate) const VAR_BATTERY_RUNTIME: &'static str = "battery.runtime";
pub(crate) const VAR_BATTERY_VOLTAGE: &'static str = "battery.voltage";
pub(crate) const VAR_BATTERY_VOLTAGE_NOMINAL: &'static str = "battery.voltage.nominal";
pub(crate) const VAR_BATTERY_TYPE: &'static str = "battery.type";
pub(crate) const VAR_DEVICE_MFR: &'static str = "device.mfr";
pub(crate) const VAR_DEVICE_MODEL: &'static str = "device.model";
pub(crate) const VAR_DEVICE_SERIAL: &'static str = "device.serial";
pub(crate) const VAR_DEVICE_TYPE: &'static str = "device.type";
pub(crate) const VAR_DRIVER_NAME: &'static str = "driver.name";
pub(crate) const VAR_DRIVER_PARAMETER_LOWBATT: &'static str = "driver.parameter.lowbatt";
pub(crate) const VAR_DRIVER_PARAMETER_OFFDELAY: &'static str = "driver.parameter.offdelay";
pub(crate) const VAR_DRIVER_PARAMETER_ONDELAY: &'static str = "driver.parameter.ondelay";
pub(crate) const VAR_DRIVER_PARAMETER_POLLFREQ: &'static str = "driver.parameter.pollfreq";
pub(crate) const VAR_DRIVER_PARAMETER_POLLINTERVAL: &'static str = "driver.parameter.pollinterval";
pub(crate) const VAR_DRIVER_PARAMETER_PORT: &'static str = "driver.parameter.port";
pub(crate) const VAR_DRIVER_PARAMETER_SYNCHRONOUS: &'static str = "driver.parameter.synchronous";
pub(crate) const VAR_DRIVER_PARAMETER_VENDORID: &'static str = "driver.parameter.vendorid";
pub(crate) const VAR_DRIVER_VERSION: &'static str = "driver.version";
pub(crate) const VAR_DRIVER_VERSION_DATA: &'static str = "driver.version.data";
pub(crate) const VAR_DRIVER_VERSION_INTERNAL: &'static str = "driver.version.internal";
pub(crate) const VAR_INPUT_TRANSFER_HIGH: &'static str = "input.transfer.high";
pub(crate) const VAR_INPUT_TRANSFER_LOW: &'static str = "input.transfer.low";
pub(crate) const VAR_INPUT_VOLTAGE: &'static str = "input.voltage";
pub(crate) const VAR_INPUT_VOLTAGE_NOMINAL: &'static str = "input.voltage.nominal";
pub(crate) const VAR_OUTPUT_FREQUENCY_NOMINAL: &'static str = "output.frequency.nominal";
pub(crate) const VAR_OUTPUT_VOLTAGE: &'static str = "output.voltage";
pub(crate) const VAR_OUTPUT_VOLTAGE_NOMINAL: &'static str = "output.voltage.nominal";
pub(crate) const VAR_UPS_BEEPER_STATUS: &'static str = "ups.beeper.status";
pub(crate) const VAR_UPS_DELAY_SHUTDOWN: &'static str = "ups.delay.shutdown";
pub(crate) const VAR_UPS_DELAY_START: &'static str = "ups.delay.start";
pub(crate) const VAR_UPS_FIRMWARE: &'static str = "ups.firmware";
pub(crate) const VAR_UPS_LOAD: &'static str = "ups.load";
pub(crate) const VAR_UPS_MFR: &'static str = "ups.mfr";
pub(crate) const VAR_UPS_MODEL: &'static str = "ups.model";
pub(crate) const VAR_UPS_REALPOWER: &'static str = "ups.realpower";
pub(crate) const VAR_UPS_REALPOWER_NOMINAL: &'static str = "ups.realpower.nominal";
pub(crate) const VAR_UPS_PRODUCTID: &'static str = "ups.productid";
pub(crate) const VAR_UPS_SERIAL: &'static str = "ups.serial";
pub(crate) const VAR_UPS_STATUS: &'static str = "ups.status";
pub(crate) const VAR_UPS_TEMPERATURE: &'static str = "ups.temperature";
pub(crate) const VAR_UPS_TIMER_SHUTDOWN: &'static str = "ups.timer.shutdown";
pub(crate) const VAR_UPS_TIMER_START: &'static str = "ups.timer.start";
pub(crate) const VAR_UPS_VENDORID: &'static str = "ups.vendorid";

pub(crate) const STATUS_ALARM: &'static str = "ALARM";
pub(crate) const STATUS_BOOST: &'static str = "BOOST";
pub(crate) const STATUS_BYPASS: &'static str = "BYPASS";
pub(crate) const STATUS_CAL: &'static str = "CAL";
pub(crate) const STATUS_CHRG: &'static str = "CHRG";
pub(crate) const STATUS_COMM: &'static str = "COMM";
pub(crate) const STATUS_DISCHRG: &'static str = "DISCHRG";
pub(crate) const STATUS_FSD: &'static str = "FSD";
pub(crate) const STATUS_LB: &'static str = "LB";
pub(crate) const STATUS_NOCOMM: &'static str = "NOCOMM";
pub(crate) const STATUS_OB: &'static str = "OB";
pub(crate) const STATUS_OFF: &'static str = "OFF";
pub(crate) const STATUS_OL: &'static str = "OL";
pub(crate) const STATUS_OVER: &'static str = "OVER";
pub(crate) const STATUS_RB: &'static str = "RB";
pub(crate) const STATUS_TEST: &'static str = "TEST";
pub(crate) const STATUS_TICK: &'static str = "TICK";
pub(crate) const STATUS_TOCK: &'static str = "TOCK";
pub(crate) const STATUS_TRIM: &'static str = "TRIM";

pub(crate) const ERR_ACCESS_DENIED: &'static str = "ACCESS-DENIED";
pub(crate) const ERR_ALREADY_ATTACHED: &'static str = "ALREADY-ATTACHED";
pub(crate) const ERR_ALREADY_SET_PASSWORD: &'static str = "ALREADY-SET-PASSWORD";
pub(crate) const ERR_ALREADY_SET_USERNAME: &'static str = "ALREADY-SET-USERNAME";
pub(crate) const ERR_CMD_NOT_SUPPORTED: &'static str = "CMD-NOT-SUPPORTED";
pub(crate) const ERR_DATE_STALE: &'static str = "DATA-STALE";
pub(crate) const ERR_DRIVER_NOT_CONNECTED: &'static str = "DRIVER-NOT-CONNECTED";
pub(crate) const ERR_FEATURE_NOT_CONFIGURED: &'static str = "FEATURE-NOT-CONFIGURED";
pub(crate) const ERR_FEATURE_NOT_SUPPORTED: &'static str = "FEATURE-NOT-SUPPORTED";
pub(crate) const ERR_INSTCMD_FAILED: &'static str = "INSTCMD-FAILED";
pub(crate) const ERR_INVALID_ARGUMENT: &'static str = "INVALID-ARGUMENT";
pub(crate) const ERR_INVALID_PASSWORD: &'static str = "INVALID-PASSWORD";
pub(crate) const ERR_INVALID_USERNAME: &'static str = "INVALID-USERNAME";
pub(crate) const ERR_INVALID_VALUE: &'static str = "INVALID-VALUE";
pub(crate) const ERR_PASSWORD_REQUIRED: &'static str = "PASSWORD-REQUIRED";
pub(crate) const ERR_READONLY: &'static str = "READONLY";
pub(crate) const ERR_SET_FAILED: &'static str = "SET-FAILED";
pub(crate) const ERR_TLS_ALREADY_ENABLED: &'static str = "TLS-ALREADY-ENABLED";
pub(crate) const ERR_TLS_NOT_ENABLED: &'static str = "TLS-NOT-ENABLED";
pub(crate) const ERR_TOO_LONG: &'static str = "TOO-LONG";
pub(crate) const ERR_UNKNOWN_COMMAND: &'static str = "UNKNOWN-COMMAND";
pub(crate) const ERR_UNKNOWN_UPS: &'static str = "UNKNOWN-UPS";
pub(crate) const ERR_USERNAME_REQUIRED: &'static str = "USERNAME-REQUIRED";
pub(crate) const ERR_VAR_NOT_SUPPORTED: &'static str = "VAR-NOT-SUPPORTED";

#[derive(Debug, Clone, PartialEq)]
pub enum UpsVariable {
  BatteryCharge(u8),
  BatteryLow(u8),
  BatteryRuntime(i32),
  BatteryVoltage(f64),
  BatteryVoltageNominal(f64),
  BatteryType(String),
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
  InputTransferHigh(i32),
  InputTransferLow(i32),
  InputVoltage(f64),
  InputVoltageNominal(f64),
  OutputFrequencyNominal(String),
  OutputVoltage(String),
  OutputVoltageNominal(String),
  UpsBeeperStatus(String),
  UpsDelayShutdown(i32),
  UpsDelayStart(i32),
  UpsFirmware(String),
  UpsLoad(u8),
  UpsMfr(String),
  UpsModel(String),
  UpsPower(f64),
  UpsPowerNominal(f64),
  UpsProductId(String),
  UpsSerial(String),
  UpsStatus(UpsStatus),
  UpsTemperature(String),
  UpsTimerShutdown(i32),
  UpsTimerStart(i32),
  UpsVendorId(String),
  Generic(String, String),
}

impl Serialize for UpsVariable {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match self {
      UpsVariable::BatteryCharge(value) => serializer.serialize_u8(*value),
      UpsVariable::BatteryLow(value) => serializer.serialize_u8(*value),
      UpsVariable::BatteryRuntime(value) => serializer.serialize_i32(*value),
      UpsVariable::BatteryVoltage(value) => serializer.serialize_f64(*value),
      UpsVariable::BatteryVoltageNominal(value) => serializer.serialize_f64(*value),
      UpsVariable::BatteryType(value) => serializer.serialize_str(&value),
      UpsVariable::DeviceMfr(value) => serializer.serialize_str(&value),
      UpsVariable::DeviceModel(value) => serializer.serialize_str(&value),
      UpsVariable::DeviceSerial(value) => serializer.serialize_str(&value),
      UpsVariable::DeviceType(value) => serializer.serialize_str(&value),
      UpsVariable::DriverName(value) => serializer.serialize_str(&value),
      UpsVariable::DriverParameterLowBatt(value) => serializer.serialize_i32(*value),
      UpsVariable::DriverParameterOffDelay(value) => serializer.serialize_i32(*value),
      UpsVariable::DriverParameterOnDelay(value) => serializer.serialize_i32(*value),
      UpsVariable::DriverParameterPollFreq(value) => serializer.serialize_i32(*value),
      UpsVariable::DriverParameterPollInterval(value) => serializer.serialize_i32(*value),
      UpsVariable::DriverParameterPort(value) => serializer.serialize_str(&value),
      UpsVariable::DriverParameterSynchronous(value) => serializer.serialize_str(&value),
      UpsVariable::DriverParameterVendorId(value) => serializer.serialize_str(&value),
      UpsVariable::DriverVersion(value) => serializer.serialize_str(&value),
      UpsVariable::DriverVersionData(value) => serializer.serialize_str(&value),
      UpsVariable::DriverVersionInternal(value) => serializer.serialize_str(&value),
      UpsVariable::InputTransferHigh(value) => serializer.serialize_i32(*value),
      UpsVariable::InputTransferLow(value) => serializer.serialize_i32(*value),
      UpsVariable::InputVoltage(value) => serializer.serialize_f64(*value),
      UpsVariable::InputVoltageNominal(value) => serializer.serialize_f64(*value),
      UpsVariable::OutputFrequencyNominal(value) => serializer.serialize_str(&value),
      UpsVariable::OutputVoltage(value) => serializer.serialize_str(&value),
      UpsVariable::OutputVoltageNominal(value) => serializer.serialize_str(&value),
      UpsVariable::UpsBeeperStatus(value) => serializer.serialize_str(&value),
      UpsVariable::UpsDelayShutdown(value) => serializer.serialize_i32(*value),
      UpsVariable::UpsDelayStart(value) => serializer.serialize_i32(*value),
      UpsVariable::UpsFirmware(value) => serializer.serialize_str(&value),
      UpsVariable::UpsLoad(value) => serializer.serialize_u8(*value),
      UpsVariable::UpsMfr(value) => serializer.serialize_str(&value),
      UpsVariable::UpsModel(value) => serializer.serialize_str(&value),
      UpsVariable::UpsPower(value) => serializer.serialize_f64(*value),
      UpsVariable::UpsPowerNominal(value) => serializer.serialize_f64(*value),
      UpsVariable::UpsProductId(value) => serializer.serialize_str(&value),
      UpsVariable::UpsSerial(value) => serializer.serialize_str(&value),
      UpsVariable::UpsStatus(value) => serializer.serialize_str(&value.to_string()),
      UpsVariable::UpsTemperature(value) => serializer.serialize_str(&value),
      UpsVariable::UpsTimerShutdown(value) => serializer.serialize_i32(*value),
      UpsVariable::UpsTimerStart(value) => serializer.serialize_i32(*value),
      UpsVariable::UpsVendorId(value) => serializer.serialize_str(&value),
      UpsVariable::Generic(_, value) => serializer.serialize_str(&value),
    }
  }
}

impl UpsVariable {
  pub fn name(&self) -> &str {
    match self {
      UpsVariable::BatteryCharge(_) => VAR_BATTERY_CHARGE,
      UpsVariable::BatteryLow(_) => VAR_BATTERY_CHARGE_LOW,
      UpsVariable::BatteryVoltage(_) => VAR_BATTERY_VOLTAGE,
      UpsVariable::BatteryVoltageNominal(_) => VAR_BATTERY_VOLTAGE_NOMINAL,
      UpsVariable::BatteryRuntime(_) => VAR_BATTERY_RUNTIME,
      UpsVariable::BatteryType(_) => VAR_BATTERY_TYPE,
      UpsVariable::DeviceMfr(_) => VAR_DEVICE_MFR,
      UpsVariable::DeviceModel(_) => VAR_DEVICE_MODEL,
      UpsVariable::DeviceSerial(_) => VAR_DEVICE_SERIAL,
      UpsVariable::DeviceType(_) => VAR_DEVICE_TYPE,
      UpsVariable::DriverName(_) => VAR_DRIVER_NAME,
      UpsVariable::DriverParameterLowBatt(_) => VAR_DRIVER_PARAMETER_LOWBATT,
      UpsVariable::DriverParameterOffDelay(_) => VAR_DRIVER_PARAMETER_OFFDELAY,
      UpsVariable::DriverParameterOnDelay(_) => VAR_DRIVER_PARAMETER_ONDELAY,
      UpsVariable::DriverParameterVendorId(_) => VAR_DRIVER_PARAMETER_VENDORID,
      UpsVariable::DriverParameterPollFreq(_) => VAR_DRIVER_PARAMETER_POLLFREQ,
      UpsVariable::DriverParameterPollInterval(_) => VAR_DRIVER_PARAMETER_POLLINTERVAL,
      UpsVariable::DriverParameterPort(_) => VAR_DRIVER_PARAMETER_PORT,
      UpsVariable::DriverParameterSynchronous(_) => VAR_DRIVER_PARAMETER_SYNCHRONOUS,
      UpsVariable::DriverVersion(_) => VAR_DRIVER_VERSION,
      UpsVariable::DriverVersionData(_) => VAR_DRIVER_VERSION_DATA,
      UpsVariable::DriverVersionInternal(_) => VAR_DRIVER_VERSION_INTERNAL,
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
      UpsVariable::UpsPower(_) => VAR_UPS_REALPOWER,
      UpsVariable::UpsPowerNominal(_) => VAR_UPS_REALPOWER_NOMINAL,
      UpsVariable::UpsProductId(_) => VAR_UPS_PRODUCTID,
      UpsVariable::UpsSerial(_) => VAR_UPS_SERIAL,
      UpsVariable::UpsStatus(_) => VAR_UPS_STATUS,
      UpsVariable::UpsTemperature(_) => VAR_UPS_TEMPERATURE,
      UpsVariable::UpsTimerShutdown(_) => VAR_UPS_TIMER_SHUTDOWN,
      UpsVariable::UpsTimerStart(_) => VAR_UPS_TIMER_START,
      UpsVariable::UpsVendorId(_) => VAR_UPS_VENDORID,
      UpsVariable::Generic(name, _) => name,
    }
  }

  pub fn value_as_string(&self) -> String {
    match self {
      UpsVariable::BatteryCharge(val) => val.to_string(),
      UpsVariable::BatteryLow(val) => val.to_string(),
      UpsVariable::BatteryRuntime(val) => val.to_string(),
      UpsVariable::BatteryVoltage(val) => val.to_string(),
      UpsVariable::BatteryVoltageNominal(val) => val.to_string(),
      UpsVariable::BatteryType(val) => val.to_string(),
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
      UpsVariable::UpsSerial(val) => val.to_string(),
      UpsVariable::UpsStatus(val) => val.to_string(),
      UpsVariable::UpsTemperature(val) => val.to_string(),
      UpsVariable::UpsTimerShutdown(val) => val.to_string(),
      UpsVariable::UpsTimerStart(val) => val.to_string(),
      UpsVariable::UpsVendorId(val) => val.to_string(),
      UpsVariable::Generic(_, val) => val.to_string(),
    }
  }
}

impl From<ParseIntError> for NutClientErrors {
  #[inline]
  fn from(value: ParseIntError) -> Self {
    Self::ParseError(value.to_string())
  }
}

impl From<ParseFloatError> for NutClientErrors {
  #[inline]
  fn from(value: ParseFloatError) -> Self {
    Self::ParseError(value.to_string())
  }
}

impl TryFrom<(&str, &str)> for UpsVariable {
  type Error = NutClientErrors;
  fn try_from(from_value: (&str, &str)) -> Result<Self, Self::Error> {
    let (name, value) = from_value;
    let ups_variable = match name {
      VAR_BATTERY_CHARGE => UpsVariable::BatteryCharge(value.parse::<u8>()?),
      VAR_BATTERY_CHARGE_LOW => UpsVariable::BatteryLow(value.parse::<u8>()?),
      VAR_BATTERY_RUNTIME => UpsVariable::BatteryRuntime(value.parse::<i32>()?),
      VAR_BATTERY_VOLTAGE => UpsVariable::BatteryVoltage(value.parse::<f64>()?),
      VAR_BATTERY_VOLTAGE_NOMINAL => UpsVariable::BatteryVoltageNominal(value.parse::<f64>()?),
      VAR_BATTERY_TYPE => UpsVariable::BatteryType(value.into()),
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
      VAR_INPUT_TRANSFER_HIGH => UpsVariable::InputTransferHigh(value.parse::<i32>()?),
      VAR_INPUT_TRANSFER_LOW => UpsVariable::InputTransferLow(value.parse::<i32>()?),
      VAR_INPUT_VOLTAGE => UpsVariable::InputVoltage(value.parse::<f64>()?),
      VAR_INPUT_VOLTAGE_NOMINAL => UpsVariable::InputVoltageNominal(value.parse::<f64>()?),
      VAR_OUTPUT_FREQUENCY_NOMINAL => UpsVariable::OutputFrequencyNominal(value.into()),
      VAR_OUTPUT_VOLTAGE => UpsVariable::OutputVoltage(value.into()),
      VAR_OUTPUT_VOLTAGE_NOMINAL => UpsVariable::OutputVoltageNominal(value.into()),
      VAR_UPS_BEEPER_STATUS => UpsVariable::UpsBeeperStatus(value.into()),
      VAR_UPS_DELAY_SHUTDOWN => UpsVariable::UpsDelayShutdown(value.parse::<i32>()?),
      VAR_UPS_DELAY_START => UpsVariable::UpsDelayStart(value.parse::<i32>()?),
      VAR_UPS_FIRMWARE => UpsVariable::UpsFirmware(value.into()),
      VAR_UPS_LOAD => UpsVariable::UpsLoad(value.parse::<u8>()?),
      VAR_UPS_MFR => UpsVariable::UpsMfr(value.into()),
      VAR_UPS_MODEL => UpsVariable::UpsModel(value.into()),
      VAR_UPS_REALPOWER => UpsVariable::UpsPower(value.parse::<f64>()?),
      VAR_UPS_REALPOWER_NOMINAL => UpsVariable::UpsPowerNominal(value.parse::<f64>()?),
      VAR_UPS_PRODUCTID => UpsVariable::UpsProductId(value.into()),
      VAR_UPS_SERIAL => UpsVariable::UpsSerial(value.into()),
      VAR_UPS_STATUS => UpsVariable::UpsStatus(value.into()),
      VAR_UPS_TEMPERATURE => UpsVariable::UpsTemperature(value.into()),
      VAR_UPS_TIMER_SHUTDOWN => UpsVariable::UpsTimerShutdown(value.parse::<i32>()?),
      VAR_UPS_TIMER_START => UpsVariable::UpsTimerStart(value.parse::<i32>()?),
      VAR_UPS_VENDORID => UpsVariable::UpsVendorId(value.into()),
      param => UpsVariable::Generic(param.to_owned(), value.to_owned()),
    };

    Ok(ups_variable)
  }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
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
    let status_text = match self {
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
    };

    f.write_str(status_text)
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
      UpsError::Unknown(value) => &value,
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
