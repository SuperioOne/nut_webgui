use serde::Serialize;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub enum UpsVariable {
  BatteryCharge(u8),
  BatteryLow(u8),
  BatteryRuntime(i32),
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
  UpsPowerNominal(u32),
  UpsProductId(String),
  UpsSerial(String),
  UpsStatus(UpsStatus),
  UpsTemperature(String),
  UpsTimerShutdown(i32),
  UpsTimerStart(i32),
  UpsVendorId(String),
  Generic(String, String),
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

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UpsError {
  // ACCESS-DENIED
  AccessDenied,
  // ALREADY-ATTACHED
  AlreadyAttached,
  // ALREADY-SET-PASSWORD
  AlreadySetPassword,
  // ALREADY-SET-USERNAME
  AlreadySetUsername,
  // CMD-NOT-SUPPORTED
  CmdNotSupported,
  // DATA-STALE
  DataStale,
  // DRIVER-NOT-CONNECTED
  DriverNotConnected,
  // FEATURE-NOT-CONFIGURED
  FeatureNotConfigured,
  // FEATURE-NOT-SUPPORTED
  FeatureNotSupported,
  // INSTCMD-FAILED
  InstCmdFailed,
  // INVALID-ARGUMENT
  InvalidArgument,
  // INVALID-PASSWORD
  InvalidPassword,
  // INVALID-USERNAME
  InvalidUsername,
  // INVALID-VALUE
  InvalidValue,
  // PASSWORD-REQUIRED
  PasswordRequired,
  // READONLY
  READONLY,
  // SET-FAILED
  SetFailed,
  // TLS-ALREADY-ENABLED
  TlsAlreadyEnabled,
  // TLS-NOT-ENABLED
  TlsNotEnabled,
  // TOO-LONG
  TooLong,
  // UNKNOWN-COMMAND
  UnknownCommand,
  // UNKNOWN-UPS
  UnknownUps,
  // USERNAME-REQUIRED
  UsernameRequired,
  // VAR-NOT-SUPPORTED
  VarNotSupported,
  Unknown(String),
}

impl From<&str> for UpsStatus {
  fn from(value: &str) -> Self {
    match value {
      "ALARM" => UpsStatus::Alarm,
      "BOOST" => UpsStatus::Boost,
      "BYPASS" => UpsStatus::Bypass,
      "CAL" => UpsStatus::Calibrating,
      "CHRG" => UpsStatus::Charging,
      "COMM" => UpsStatus::COMM,
      "DISCHRG" => UpsStatus::Discharging,
      "FSD" => UpsStatus::ForcedShutdown,
      "LB" => UpsStatus::LowBattery,
      "NOCOMM" => UpsStatus::NoCOMM,
      "OB" => UpsStatus::OnBattery,
      "OFF" => UpsStatus::Offline,
      "OL" => UpsStatus::Online,
      "OVER" => UpsStatus::Overloaded,
      "RB" => UpsStatus::ReplaceBattery,
      "TEST" => UpsStatus::Test,
      "TICK" => UpsStatus::Tick,
      "TOCK" => UpsStatus::Tock,
      "TRIM" => UpsStatus::Trim,
      _ => UpsStatus::Unknown(String::from(value)),
    }
  }
}

impl Display for UpsStatus {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let status_text = match self {
      UpsStatus::Alarm => "ALARM",
      UpsStatus::Boost => "BOOST",
      UpsStatus::Bypass => "BYPASS",
      UpsStatus::Calibrating => "CAL",
      UpsStatus::Charging => "CHRG",
      UpsStatus::COMM => "COMM",
      UpsStatus::Discharging => "DISCHRG",
      UpsStatus::ForcedShutdown => "FSD",
      UpsStatus::LowBattery => "LB",
      UpsStatus::NoCOMM => "NOCOMM",
      UpsStatus::OnBattery => "OB",
      UpsStatus::Offline => "OFF",
      UpsStatus::Online => "OL",
      UpsStatus::Overloaded => "OVER",
      UpsStatus::ReplaceBattery => "RB",
      UpsStatus::Test => "TEST",
      UpsStatus::Tick => "TICK",
      UpsStatus::Tock => "TOCK",
      UpsStatus::Trim => "TRIM",
      UpsStatus::Unknown(value) => value,
    };

    f.write_str(status_text)
  }
}

impl Display for UpsError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let error_text = match self {
      UpsError::AccessDenied => "ACCESS-DENIED",
      UpsError::AlreadyAttached => "ALREADY-ATTACHED",
      UpsError::AlreadySetPassword => "ALREADY-SET-PASSWORD",
      UpsError::AlreadySetUsername => "ALREADY-SET-USERNAME",
      UpsError::CmdNotSupported => "CMD-NOT-SUPPORTED",
      UpsError::DataStale => "DATA-STALE",
      UpsError::DriverNotConnected => "DRIVER-NOT-CONNECTED",
      UpsError::FeatureNotConfigured => "FEATURE-NOT-CONFIGURED",
      UpsError::FeatureNotSupported => "FEATURE-NOT-SUPPORTED",
      UpsError::InstCmdFailed => "INSTCMD-FAILED",
      UpsError::InvalidArgument => "INVALID-ARGUMENT",
      UpsError::InvalidPassword => "INVALID-PASSWORD",
      UpsError::InvalidUsername => "INVALID-USERNAME",
      UpsError::InvalidValue => "INVALID-VALUE",
      UpsError::PasswordRequired => "PASSWORD-REQUIRED",
      UpsError::READONLY => "READONLY",
      UpsError::SetFailed => "SET-FAILED",
      UpsError::TlsAlreadyEnabled => "TLS-ALREADY-ENABLED",
      UpsError::TlsNotEnabled => "TLS-NOT-ENABLED",
      UpsError::TooLong => "TOO-LONG",
      UpsError::UnknownCommand => "UNKNOWN-COMMAND",
      UpsError::UnknownUps => "UNKNOWN-UPS",
      UpsError::UsernameRequired => "USERNAME-REQUIRED",
      UpsError::VarNotSupported => "VAR-NOT-SUPPORTED",
      UpsError::Unknown(value) => value.as_str(),
    };

    f.write_str(error_text)
  }
}

impl From<&str> for UpsError {
  fn from(value: &str) -> Self {
    match value {
      "ACCESS-DENIED" => UpsError::AccessDenied,
      "ALREADY-ATTACHED" => UpsError::AlreadyAttached,
      "ALREADY-SET-PASSWORD" => UpsError::AlreadySetPassword,
      "ALREADY-SET-USERNAME" => UpsError::AlreadySetUsername,
      "CMD-NOT-SUPPORTED" => UpsError::CmdNotSupported,
      "DATA-STALE" => UpsError::DataStale,
      "DRIVER-NOT-CONNECTED" => UpsError::DriverNotConnected,
      "FEATURE-NOT-CONFIGURED" => UpsError::FeatureNotConfigured,
      "FEATURE-NOT-SUPPORTED" => UpsError::FeatureNotSupported,
      "INSTCMD-FAILED" => UpsError::InstCmdFailed,
      "INVALID-ARGUMENT" => UpsError::InvalidArgument,
      "INVALID-PASSWORD" => UpsError::InvalidPassword,
      "INVALID-USERNAME" => UpsError::InvalidUsername,
      "INVALID-VALUE" => UpsError::InvalidValue,
      "PASSWORD-REQUIRED" => UpsError::PasswordRequired,
      "READONLY" => UpsError::READONLY,
      "SET-FAILED" => UpsError::SetFailed,
      "TLS-ALREADY-ENABLED" => UpsError::TlsAlreadyEnabled,
      "TLS-NOT-ENABLED" => UpsError::TlsNotEnabled,
      "TOO-LONG" => UpsError::TooLong,
      "UNKNOWN-COMMAND" => UpsError::UnknownCommand,
      "UNKNOWN-UPS" => UpsError::UnknownUps,
      "USERNAME-REQUIRED" => UpsError::UsernameRequired,
      "VAR-NOT-SUPPORTED" => UpsError::VarNotSupported,
      _ => UpsError::Unknown(value.to_owned()),
    }
  }
}

impl UpsVariable {
  pub fn name(&self) -> String {
    match self {
      UpsVariable::BatteryCharge(_) => String::from("battery.charge"),
      UpsVariable::BatteryLow(_) => String::from("battery.charge.low"),
      UpsVariable::BatteryRuntime(_) => String::from("battery.runtime"),
      UpsVariable::BatteryType(_) => String::from("battery.type"),
      UpsVariable::DeviceMfr(_) => String::from("device.mfr"),
      UpsVariable::DeviceModel(_) => String::from("device.model"),
      UpsVariable::DeviceSerial(_) => String::from("device.serial"),
      UpsVariable::DeviceType(_) => String::from("device.type"),
      UpsVariable::DriverName(_) => String::from("driver.name"),
      UpsVariable::DriverParameterLowBatt(_) => String::from("driver.parameter.lowbatt"),
      UpsVariable::DriverParameterOffDelay(_) => String::from("driver.parameter.offdelay"),
      UpsVariable::DriverParameterOnDelay(_) => String::from("driver.parameter.ondelay"),
      UpsVariable::DriverParameterPollFreq(_) => String::from("driver.parameter.pollfreq"),
      UpsVariable::DriverParameterPollInterval(_) => String::from("driver.parameter.pollinterval"),
      UpsVariable::DriverParameterPort(_) => String::from("driver.parameter.port"),
      UpsVariable::DriverParameterSynchronous(_) => String::from("driver.parameter.synchronous"),
      UpsVariable::DriverParameterVendorId(_) => String::from("driver.parameter.vendorid"),
      UpsVariable::DriverVersion(_) => String::from("driver.version"),
      UpsVariable::DriverVersionData(_) => String::from("driver.version.data"),
      UpsVariable::DriverVersionInternal(_) => String::from("driver.version.internal"),
      UpsVariable::InputTransferHigh(_) => String::from("input.transfer.high"),
      UpsVariable::InputTransferLow(_) => String::from("input.transfer.low"),
      UpsVariable::OutputFrequencyNominal(_) => String::from("output.frequency.nominal"),
      UpsVariable::OutputVoltage(_) => String::from("output.voltage"),
      UpsVariable::OutputVoltageNominal(_) => String::from("output.voltage.nominal"),
      UpsVariable::UpsBeeperStatus(_) => String::from("ups.beeper.status"),
      UpsVariable::UpsDelayShutdown(_) => String::from("ups.delay.shutdown"),
      UpsVariable::UpsDelayStart(_) => String::from("ups.delay.start"),
      UpsVariable::UpsFirmware(_) => String::from("ups.firmware"),
      UpsVariable::UpsLoad(_) => String::from("ups.load"),
      UpsVariable::UpsMfr(_) => String::from("ups.mfr"),
      UpsVariable::UpsModel(_) => String::from("ups.model"),
      UpsVariable::UpsPowerNominal(_) => String::from("ups.realpower.nominal"),
      UpsVariable::UpsProductId(_) => String::from("ups.productid"),
      UpsVariable::UpsSerial(_) => String::from("ups.serial"),
      UpsVariable::UpsStatus(_) => String::from("ups.status"),
      UpsVariable::UpsTemperature(_) => String::from("ups.temperature"),
      UpsVariable::UpsTimerShutdown(_) => String::from("ups.timer.shutdown"),
      UpsVariable::UpsTimerStart(_) => String::from("ups.timer.start"),
      UpsVariable::UpsVendorId(_) => String::from("ups.vendorid"),
      UpsVariable::Generic(name, _) => name.clone(),
    }
  }

  pub fn value_as_string(&self) -> String {
    match self {
      UpsVariable::BatteryCharge(val) => val.to_string(),
      UpsVariable::BatteryLow(val) => val.to_string(),
      UpsVariable::BatteryRuntime(val) => val.to_string(),
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

impl TryFrom<(&str, &str)> for UpsVariable {
  type Error = ParseIntError;
  fn try_from(from_value: (&str, &str)) -> Result<Self, Self::Error> {
    let (name, value) = from_value;
    let ups_variable = match name {
      "battery.charge" => UpsVariable::BatteryCharge(value.parse::<u8>()?),
      "battery.charge.low" => UpsVariable::BatteryLow(value.parse::<u8>()?),
      "battery.runtime" => UpsVariable::BatteryRuntime(value.parse::<i32>()?),
      "battery.type" => UpsVariable::BatteryType(value.into()),
      "device.mfr" => UpsVariable::DeviceMfr(value.into()),
      "device.model" => UpsVariable::DeviceModel(value.into()),
      "device.serial" => UpsVariable::DeviceSerial(value.into()),
      "device.type" => UpsVariable::DeviceType(value.into()),
      "driver.name" => UpsVariable::DriverName(value.into()),
      "driver.parameter.lowbatt" => UpsVariable::DriverParameterLowBatt(value.parse::<i32>()?),
      "driver.parameter.offdelay" => UpsVariable::DriverParameterOffDelay(value.parse::<i32>()?),
      "driver.parameter.ondelay" => UpsVariable::DriverParameterOnDelay(value.parse::<i32>()?),
      "driver.parameter.pollfreq" => UpsVariable::DriverParameterPollFreq(value.parse::<i32>()?),
      "driver.parameter.pollinterval" => {
        UpsVariable::DriverParameterPollInterval(value.parse::<i32>()?)
      }
      "driver.parameter.port" => UpsVariable::DriverParameterPort(value.into()),
      "driver.parameter.synchronous" => UpsVariable::DriverParameterSynchronous(value.into()),
      "driver.parameter.vendorid" => UpsVariable::DriverParameterVendorId(value.into()),
      "driver.version" => UpsVariable::DriverVersion(value.into()),
      "driver.version.data" => UpsVariable::DriverVersionData(value.into()),
      "driver.version.internal" => UpsVariable::DriverVersionInternal(value.into()),
      "input.transfer.high" => UpsVariable::InputTransferHigh(value.parse::<i32>()?),
      "input.transfer.low" => UpsVariable::InputTransferLow(value.parse::<i32>()?),
      "output.frequency.nominal" => UpsVariable::OutputFrequencyNominal(value.into()),
      "output.voltage" => UpsVariable::OutputVoltage(value.into()),
      "output.voltage.nominal" => UpsVariable::OutputVoltageNominal(value.into()),
      "ups.beeper.status" => UpsVariable::UpsBeeperStatus(value.into()),
      "ups.delay.shutdown" => UpsVariable::UpsDelayShutdown(value.parse::<i32>()?),
      "ups.delay.start" => UpsVariable::UpsDelayStart(value.parse::<i32>()?),
      "ups.firmware" => UpsVariable::UpsFirmware(value.into()),
      "ups.load" => UpsVariable::UpsLoad(value.parse::<u8>()?),
      "ups.mfr" => UpsVariable::UpsMfr(value.into()),
      "ups.model" => UpsVariable::UpsModel(value.into()),
      "ups.realpower.nominal" => UpsVariable::UpsPowerNominal(value.parse::<u32>()?),
      "ups.productid" => UpsVariable::UpsProductId(value.into()),
      "ups.serial" => UpsVariable::UpsSerial(value.into()),
      "ups.status" => UpsVariable::UpsStatus(value.into()),
      "ups.temperature" => UpsVariable::UpsTemperature(value.into()),
      "ups.timer.shutdown" => UpsVariable::UpsTimerShutdown(value.parse::<i32>()?),
      "ups.timer.start" => UpsVariable::UpsTimerStart(value.parse::<i32>()?),
      "ups.vendorid" => UpsVariable::UpsVendorId(value.into()),
      param => UpsVariable::Generic(param.into(), value.into()),
    };

    Ok(ups_variable)
  }
}
