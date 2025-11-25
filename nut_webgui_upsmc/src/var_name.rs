use super::internal::{Repr, ascii_rules::NutAsciiText};
use crate::error::VarNameParseError;
use core::borrow::Borrow;

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
  StandardName,
  (AMBIENT_HUMIDITY                                 ,AmbientHumidity                             ,"ambient.humidity");
  (AMBIENT_HUMIDITY_ALARM_MAXIMUM                   ,AmbientHumidityAlarmMaximum                 ,"ambient.humidity.alarm.maximum");
  (AMBIENT_HUMIDITY_ALARM_MINIMUM                   ,AmbientHumidityAlarmMinimum                 ,"ambient.humidity.alarm.minimum");
  (AMBIENT_HUMIDITY_HIGH                            ,AmbientHumidityHigh                         ,"ambient.humidity.high");
  (AMBIENT_HUMIDITY_HIGH_CRITICAL                   ,AmbientHumidityHighCritical                 ,"ambient.humidity.high.critical");
  (AMBIENT_HUMIDITY_HIGH_WARNING                    ,AmbientHumidityHighWarning                  ,"ambient.humidity.high.warning");
  (AMBIENT_HUMIDITY_LOW                             ,AmbientHumidityLow                          ,"ambient.humidity.low");
  (AMBIENT_HUMIDITY_LOW_CRITICAL                    ,AmbientHumidityLowCritical                  ,"ambient.humidity.low.critical");
  (AMBIENT_HUMIDITY_LOW_WARNING                     ,AmbientHumidityLowWarning                   ,"ambient.humidity.low.warning");
  (AMBIENT_HUMIDITY_STATUS                          ,AmbientHumidityStatus                       ,"ambient.humidity.status");
  (AMBIENT_PRESENT                                  ,AmbientPresent                              ,"ambient.present");
  (AMBIENT_TEMPERATURE                              ,AmbientTemperature                          ,"ambient.temperature");
  (AMBIENT_TEMPERATURE_ALARM_MAXIMUM                ,AmbientTemperatureAlarmMaximum              ,"ambient.temperature.alarm.maximum");
  (AMBIENT_TEMPERATURE_ALARM_MINIMUM                ,AmbientTemperatureAlarmMinimum              ,"ambient.temperature.alarm.minimum");
  (AMBIENT_TEMPERATURE_HIGH                         ,AmbientTemperatureHigh                      ,"ambient.temperature.high");
  (AMBIENT_TEMPERATURE_HIGH_CRITICAL                ,AmbientTemperatureHighCritical              ,"ambient.temperature.high.critical");
  (AMBIENT_TEMPERATURE_HIGH_WARNING                 ,AmbientTemperatureHighWarning               ,"ambient.temperature.high.warning");
  (AMBIENT_TEMPERATURE_LOW                          ,AmbientTemperatureLow                       ,"ambient.temperature.low");
  (AMBIENT_TEMPERATURE_LOW_CRITICAL                 ,AmbientTemperatureLowCritical               ,"ambient.temperature.low.critical");
  (AMBIENT_TEMPERATURE_LOW_WARNING                  ,AmbientTemperatureLowWarning                ,"ambient.temperature.low.warning");
  (AMBIENT_TEMPERATURE_STATUS                       ,AmbientTemperatureStatus                    ,"ambient.temperature.status");
  (BATTERY_ALARM_THRESHOLD                          ,BatteryAlarmThreshold                       ,"battery.alarm.threshold");
  (BATTERY_CAPACITY                                 ,BatteryCapacity                             ,"battery.capacity");
  (BATTERY_CAPACITY_NOMINAL                         ,BatteryCapacityNominal                      ,"battery.capacity.nominal");
  (BATTERY_CHARGE                                   ,BatteryCharge                               ,"battery.charge");
  (BATTERY_CHARGE_LOW                               ,BatteryChargeLow                            ,"battery.charge.low");
  (BATTERY_CHARGE_RESTART                           ,BatteryChargeRestart                        ,"battery.charge.restart");
  (BATTERY_CHARGER_STATUS                           ,BatteryChargerStatus                        ,"battery.charger.status");
  (BATTERY_CHARGE_WARNING                           ,BatteryChargeWarning                        ,"battery.charge.warning");
  (BATTERY_CHEMISTRY                                ,BatteryChemistry                            ,"battery.chemistry");
  (BATTERY_CURRENT                                  ,BatteryCurrent                              ,"battery.current");
  (BATTERY_CURRENT_MAXIMUM                          ,BatteryCurrentMaximum                       ,"battery.current.maximum");
  (BATTERY_CURRENT_MINIMUM                          ,BatteryCurrentMinimum                       ,"battery.current.minimum");
  (BATTERY_CURRENT_TOTAL                            ,BatteryCurrentTotal                         ,"battery.current.total");
  (BATTERY_DATE                                     ,BatteryDate                                 ,"battery.date");
  (BATTERY_DATE_MAINTENANCE                         ,BatteryDateMaintenance                      ,"battery.date.maintenance");
  (BATTERY_ENERGYSAVE                               ,BatteryEnergysave                           ,"battery.energysave");
  (BATTERY_ENERGYSAVE_DELAY                         ,BatteryEnergysaveDelay                      ,"battery.energysave.delay");
  (BATTERY_ENERGYSAVE_LOAD                          ,BatteryEnergysaveLoad                       ,"battery.energysave.load");
  (BATTERY_LOWBATT                                  ,BatteryLowbatt                              ,"battery.lowbatt");
  (BATTERY_MFR_DATE                                 ,BatteryMfrDate                              ,"battery.mfr.date");
  (BATTERY_PACKS                                    ,BatteryPacks                                ,"battery.packs");
  (BATTERY_PACKS_BAD                                ,BatteryPacksBad                             ,"battery.packs.bad");
  (BATTERY_PROTECTION                               ,BatteryProtection                           ,"battery.protection");
  (BATTERY_RUNTIME                                  ,BatteryRuntime                              ,"battery.runtime");
  (BATTERY_RUNTIME_ELAPSED                          ,BatteryRuntimeElapsed                       ,"battery.runtime.elapsed");
  (BATTERY_RUNTIME_LOW                              ,BatteryRuntimeLow                           ,"battery.runtime.low");
  (BATTERY_TEMPERATURE                              ,BatteryTemperature                          ,"battery.temperature");
  (BATTERY_TEST_STATUS                              ,BatteryTestStatus                           ,"battery.test.status");
  (BATTERY_TYPE                                     ,BatteryType                                 ,"battery.type");
  (BATTERY_VOLTAGE                                  ,BatteryVoltage                              ,"battery.voltage");
  (BATTERY_VOLTAGE_HIGH                             ,BatteryVoltageHigh                          ,"battery.voltage.high");
  (BATTERY_VOLTAGE_LOW                              ,BatteryVoltageLow                           ,"battery.voltage.low");
  (BATTERY_VOLTAGE_MAXIMUM                          ,BatteryVoltageMaximum                       ,"battery.voltage.maximum");
  (BATTERY_VOLTAGE_MINIMUM                          ,BatteryVoltageMinimum                       ,"battery.voltage.minimum");
  (BATTERY_VOLTAGE_NOMINAL                          ,BatteryVoltageNominal                       ,"battery.voltage.nominal");
  (DEVICE_CONTACT                                   ,DeviceContact                               ,"device.contact");
  (DEVICE_DESCRIPTION                               ,DeviceDescription                           ,"device.description");
  (DEVICE_LOCATION                                  ,DeviceLocation                              ,"device.location");
  (DEVICE_MACADDR                                   ,DeviceMacaddr                               ,"device.macaddr");
  (DEVICE_MFR                                       ,DeviceMfr                                   ,"device.mfr");
  (DEVICE_MODEL                                     ,DeviceModel                                 ,"device.model");
  (DEVICE_PART                                      ,DevicePart                                  ,"device.part");
  (DEVICE_REVISION                                  ,DeviceRevision                              ,"device.revision");
  (DEVICE_SERIAL                                    ,DeviceSerial                                ,"device.serial");
  (DEVICE_TYPE                                      ,DeviceType                                  ,"device.type");
  (DRIVER_FLAG_ALLOW_KILLPOWER                      ,DriverFlagAllowKillpower                    ,"driver.flag.allow_killpower");
  (DRIVER_FLAG_IGNORELB                             ,DriverFlagIgnorelb                          ,"driver.flag.ignorelb");
  (DRIVER_FLAG_IGNOREOFF                            ,DriverFlagIgnoreoff                         ,"driver.flag.ignoreoff");
  (DRIVER_FLAG_MAXREPORT                            ,DriverFlagMaxreport                         ,"driver.flag.maxreport");
  (DRIVER_FLAG_NOLOCK                               ,DriverFlagNolock                            ,"driver.flag.nolock");
  (DRIVER_FLAG_NORATING                             ,DriverFlagNorating                          ,"driver.flag.norating");
  (DRIVER_FLAG_NOSCANLANGID                         ,DriverFlagNoscanlangid                      ,"driver.flag.noscanlangid");
  (DRIVER_FLAG_NOVENDOR                             ,DriverFlagNovendor                          ,"driver.flag.novendor");
  (DRIVER_FLAG_POLLONLY                             ,DriverFlagPollonly                          ,"driver.flag.pollonly");
  (DRIVER_NAME                                      ,DriverName                                  ,"driver.name");
  (DRIVER_PARAMETER_ALARM_CONTROL                   ,DriverParameterAlarmControl                 ,"driver.parameter.alarm_control");
  (DRIVER_PARAMETER_BATTVOLTMULT                    ,DriverParameterBattvoltmult                 ,"driver.parameter.battvoltmult");
  (DRIVER_PARAMETER_BATTVOLTS                       ,DriverParameterBattvolts                    ,"driver.parameter.battvolts");
  (DRIVER_PARAMETER_BAUD_RATE                       ,DriverParameterBaudRate                     ,"driver.parameter.baud_rate");
  (DRIVER_PARAMETER_BAUDRATE                        ,DriverParameterBaudrate                     ,"driver.parameter.baudrate");
  (DRIVER_PARAMETER_BUS                             ,DriverParameterBus                          ,"driver.parameter.bus");
  (DRIVER_PARAMETER_BYPASS_FORBIDDING               ,DriverParameterBypassForbidding             ,"driver.parameter.bypass_forbidding");
  (DRIVER_PARAMETER_CABLE                           ,DriverParameterCable                        ,"driver.parameter.cable");
  (DRIVER_PARAMETER_CABLEPOWER                      ,DriverParameterCablepower                   ,"driver.parameter.cablepower");
  (DRIVER_PARAMETER_CHARGETIME                      ,DriverParameterChargetime                   ,"driver.parameter.chargetime");
  (DRIVER_PARAMETER_DEFAULT_BATTERY_VOLTAGE         ,DriverParameterDefaultBatteryVoltage        ,"driver.parameter.default.battery.voltage");
  (DRIVER_PARAMETER_DEFAULT_BATTERY_VOLTAGE_HIGH    ,DriverParameterDefaultBatteryVoltageHigh    ,"driver.parameter.default.battery.voltage.high");
  (DRIVER_PARAMETER_DEFAULT_BATTERY_VOLTAGE_LOW     ,DriverParameterDefaultBatteryVoltageLow     ,"driver.parameter.default.battery.voltage.low");
  (DRIVER_PARAMETER_DEFAULT_BATTERY_VOLTAGE_NOMINAL ,DriverParameterDefaultBatteryVoltageNominal ,"driver.parameter.default.battery.voltage.nominal");
  (DRIVER_PARAMETER_DEFAULT_INPUT_VOLTAGE           ,DriverParameterDefaultInputVoltage          ,"driver.parameter.default.input.voltage");
  (DRIVER_PARAMETER_DEFAULT_INPUT_VOLTAGE_NOMINAL   ,DriverParameterDefaultInputVoltageNominal   ,"driver.parameter.default.input.voltage.nominal");
  (DRIVER_PARAMETER_DEVICE                          ,DriverParameterDevice                       ,"driver.parameter.device");
  (DRIVER_PARAMETER_FREQUENCY                       ,DriverParameterFrequency                    ,"driver.parameter.frequency");
  (DRIVER_PARAMETER_IDLELOAD                        ,DriverParameterIdleload                     ,"driver.parameter.idleload");
  (DRIVER_PARAMETER_LANGID_FIX                      ,DriverParameterLangidFix                    ,"driver.parameter.langid_fix");
  (DRIVER_PARAMETER_LIMITED_RUNTIME_ON_BATTERY      ,DriverParameterLimitedRuntimeOnBattery      ,"driver.parameter.limited_runtime_on_battery");
  (DRIVER_PARAMETER_LINEVOLTAGE                     ,DriverParameterLinevoltage                  ,"driver.parameter.linevoltage");
  (DRIVER_PARAMETER_LOWBATT                         ,DriverParameterLowbatt                      ,"driver.parameter.lowbatt");
  (DRIVER_PARAMETER_LOWBATT_PCT                     ,DriverParameterLowbattPct                   ,"driver.parameter.lowbatt_pct");
  (DRIVER_PARAMETER_LOWBATTVOLT                     ,DriverParameterLowbattvolt                  ,"driver.parameter.lowbattvolt");
  (DRIVER_PARAMETER_MANUFACTURER                    ,DriverParameterManufacturer                 ,"driver.parameter.manufacturer");
  (DRIVER_PARAMETER_MFR                             ,DriverParameterMfr                          ,"driver.parameter.mfr");
  (DRIVER_PARAMETER_MIBS                            ,DriverParameterMibs                         ,"driver.parameter.mibs");
  (DRIVER_PARAMETER_MODEL                           ,DriverParameterModel                        ,"driver.parameter.model");
  (DRIVER_PARAMETER_MODELNAME                       ,DriverParameterModelname                    ,"driver.parameter.modelname");
  (DRIVER_PARAMETER_NOTIFICATION                    ,DriverParameterNotification                 ,"driver.parameter.notification");
  (DRIVER_PARAMETER_OFFDELAY                        ,DriverParameterOffdelay                     ,"driver.parameter.offdelay");
  (DRIVER_PARAMETER_ONDELAY                         ,DriverParameterOndelay                      ,"driver.parameter.ondelay");
  (DRIVER_PARAMETER_OVERRIDE_BATTERY_CHARGE_LOW     ,DriverParameterOverrideBatteryChargeLow     ,"driver.parameter.override.battery.charge.low");
  (DRIVER_PARAMETER_OVERRIDE_BATTERY_CHARGE_WARNING ,DriverParameterOverrideBatteryChargeWarning ,"driver.parameter.override.battery.charge.warning");
  (DRIVER_PARAMETER_OVERRIDE_BATTERY_VOLTAGE_NOMINAL,DriverParameterOverrideBatteryVoltageNominal,"driver.parameter.override.battery.voltage.nominal");
  (DRIVER_PARAMETER_POLLFREQ                        ,DriverParameterPollfreq                     ,"driver.parameter.pollfreq");
  (DRIVER_PARAMETER_POLLINTERVAL                    ,DriverParameterPollinterval                 ,"driver.parameter.pollinterval");
  (DRIVER_PARAMETER_PORT                            ,DriverParameterPort                         ,"driver.parameter.port");
  (DRIVER_PARAMETER_PRODUCT                         ,DriverParameterProduct                      ,"driver.parameter.product");
  (DRIVER_PARAMETER_PRODUCTID                       ,DriverParameterProductid                    ,"driver.parameter.productid");
  (DRIVER_PARAMETER_PROTOCOL                        ,DriverParameterProtocol                     ,"driver.parameter.protocol");
  (DRIVER_PARAMETER_RUNTIMECAL                      ,DriverParameterRuntimecal                   ,"driver.parameter.runtimecal");
  (DRIVER_PARAMETER_SDTYPE                          ,DriverParameterSdtype                       ,"driver.parameter.sdtype");
  (DRIVER_PARAMETER_SERIAL                          ,DriverParameterSerial                       ,"driver.parameter.serial");
  (DRIVER_PARAMETER_SERIALNUMBER                    ,DriverParameterSerialnumber                 ,"driver.parameter.serialnumber");
  (DRIVER_PARAMETER_SHUTDOWN_DELAY                  ,DriverParameterShutdownDelay                ,"driver.parameter.shutdown_delay");
  (DRIVER_PARAMETER_SNMP_RETRIES                    ,DriverParameterSnmpRetries                  ,"driver.parameter.snmp_retries");
  (DRIVER_PARAMETER_SNMP_TIMEOUT                    ,DriverParameterSnmpTimeout                  ,"driver.parameter.snmp_timeout");
  (DRIVER_PARAMETER_SNMP_VERSION                    ,DriverParameterSnmpVersion                  ,"driver.parameter.snmp_version");
  (DRIVER_PARAMETER_SUBDRIVER                       ,DriverParameterSubdriver                    ,"driver.parameter.subdriver");
  (DRIVER_PARAMETER_SYNCHRONOUS                     ,DriverParameterSynchronous                  ,"driver.parameter.synchronous");
  (DRIVER_PARAMETER_TYPE                            ,DriverParameterType                         ,"driver.parameter.type");
  (DRIVER_PARAMETER_UPSTYPE                         ,DriverParameterUpstype                      ,"driver.parameter.upstype");
  (DRIVER_PARAMETER_VENDOR                          ,DriverParameterVendor                       ,"driver.parameter.vendor");
  (DRIVER_PARAMETER_VENDORID                        ,DriverParameterVendorid                     ,"driver.parameter.vendorid");
  (DRIVER_PARAMETER_VOLTAGE                         ,DriverParameterVoltage                      ,"driver.parameter.voltage");
  (DRIVER_STATE                                     ,DriverState                                 ,"driver.state");
  (DRIVER_VERSION                                   ,DriverVersion                               ,"driver.version");
  (DRIVER_VERSION_DATA                              ,DriverVersionData                           ,"driver.version.data");
  (DRIVER_VERSION_INTERNAL                          ,DriverVersionInternal                       ,"driver.version.internal");
  (DRIVER_VERSION_USB                               ,DriverVersionUsb                            ,"driver.version.usb");
  (INPUT_BYPASS_CURRENT                             ,InputBypassCurrent                          ,"input.bypass.current");
  (INPUT_BYPASS_FREQUENCY                           ,InputBypassFrequency                        ,"input.bypass.frequency");
  (INPUT_BYPASS_FREQUENCY_NOMINAL                   ,InputBypassFrequencyNominal                 ,"input.bypass.frequency.nominal");
  (INPUT_BYPASS_PHASES                              ,InputBypassPhases                           ,"input.bypass.phases");
  (INPUT_BYPASS_VOLTAGE                             ,InputBypassVoltage                          ,"input.bypass.voltage");
  (INPUT_COUNT                                      ,InputCount                                  ,"input.count");
  (INPUT_CURRENT                                    ,InputCurrent                                ,"input.current");
  (INPUT_CURRENT_HIGH_CRITICAL                      ,InputCurrentHighCritical                    ,"input.current.high.critical");
  (INPUT_CURRENT_HIGH_WARNING                       ,InputCurrentHighWarning                     ,"input.current.high.warning");
  (INPUT_CURRENT_LOW_WARNING                        ,InputCurrentLowWarning                      ,"input.current.low.warning");
  (INPUT_CURRENT_NOMINAL                            ,InputCurrentNominal                         ,"input.current.nominal");
  (INPUT_CURRENT_STATUS                             ,InputCurrentStatus                          ,"input.current.status");
  (INPUT_FREQUENCY                                  ,InputFrequency                              ,"input.frequency");
  (INPUT_FREQUENCY_EXTENDED                         ,InputFrequencyExtended                      ,"input.frequency.extended");
  (INPUT_FREQUENCY_HIGH                             ,InputFrequencyHigh                          ,"input.frequency.high");
  (INPUT_FREQUENCY_LOW                              ,InputFrequencyLow                           ,"input.frequency.low");
  (INPUT_FREQUENCY_NOMINAL                          ,InputFrequencyNominal                       ,"input.frequency.nominal");
  (INPUT_FREQUENCY_STATUS                           ,InputFrequencyStatus                        ,"input.frequency.status");
  (INPUT_LOAD                                       ,InputLoad                                   ,"input.load");
  (INPUT_PHASES                                     ,InputPhases                                 ,"input.phases");
  (INPUT_POWER                                      ,InputPower                                  ,"input.power");
  (INPUT_POWERFACTOR                                ,InputPowerfactor                            ,"input.powerfactor");
  (INPUT_QUALITY                                    ,InputQuality                                ,"input.quality");
  (INPUT_REALPOWER                                  ,InputRealpower                              ,"input.realpower");
  (INPUT_SENSITIVITY                                ,InputSensitivity                            ,"input.sensitivity");
  (INPUT_SOURCE                                     ,InputSource                                 ,"input.source");
  (INPUT_SOURCE_PREFERRED                           ,InputSourcePreferred                        ,"input.source.preferred");
  (INPUT_TRANSFER_BOOST_HIGH                        ,InputTransferBoostHigh                      ,"input.transfer.boost.high");
  (INPUT_TRANSFER_BOOST_LOW                         ,InputTransferBoostLow                       ,"input.transfer.boost.low");
  (INPUT_TRANSFER_DELAY                             ,InputTransferDelay                          ,"input.transfer.delay");
  (INPUT_TRANSFER_HIGH                              ,InputTransferHigh                           ,"input.transfer.high");
  (INPUT_TRANSFER_HIGH_MAX                          ,InputTransferHighMax                        ,"input.transfer.high.max");
  (INPUT_TRANSFER_HIGH_MIN                          ,InputTransferHighMin                        ,"input.transfer.high.min");
  (INPUT_TRANSFER_LOW                               ,InputTransferLow                            ,"input.transfer.low");
  (INPUT_TRANSFER_LOW_MAX                           ,InputTransferLowMax                         ,"input.transfer.low.max");
  (INPUT_TRANSFER_LOW_MIN                           ,InputTransferLowMin                         ,"input.transfer.low.min");
  (INPUT_TRANSFER_REASON                            ,InputTransferReason                         ,"input.transfer.reason");
  (INPUT_TRANSFER_TRIM_HIGH                         ,InputTransferTrimHigh                       ,"input.transfer.trim.high");
  (INPUT_TRANSFER_TRIM_LOW                          ,InputTransferTrimLow                        ,"input.transfer.trim.low");
  (INPUT_VOLTAGE                                    ,InputVoltage                                ,"input.voltage");
  (INPUT_VOLTAGE_EXTENDED                           ,InputVoltageExtended                        ,"input.voltage.extended");
  (INPUT_VOLTAGE_FAULT                              ,InputVoltageFault                           ,"input.voltage.fault");
  (INPUT_VOLTAGE_HIGH_CRITICAL                      ,InputVoltageHighCritical                    ,"input.voltage.high.critical");
  (INPUT_VOLTAGE_HIGH_WARNING                       ,InputVoltageHighWarning                     ,"input.voltage.high.warning");
  (INPUT_VOLTAGE_LOW_CRITICAL                       ,InputVoltageLowCritical                     ,"input.voltage.low.critical");
  (INPUT_VOLTAGE_LOW_WARNING                        ,InputVoltageLowWarning                      ,"input.voltage.low.warning");
  (INPUT_VOLTAGE_MAXIMUM                            ,InputVoltageMaximum                         ,"input.voltage.maximum");
  (INPUT_VOLTAGE_MINIMUM                            ,InputVoltageMinimum                         ,"input.voltage.minimum");
  (INPUT_VOLTAGE_NOMINAL                            ,InputVoltageNominal                         ,"input.voltage.nominal");
  (INPUT_VOLTAGE_STATUS                             ,InputVoltageStatus                          ,"input.voltage.status");
  (OUTLET_COUNT                                     ,OutletCount                                 ,"outlet.count");
  (OUTLET_CURRENT                                   ,OutletCurrent                               ,"outlet.current");
  (OUTLET_DESC                                      ,OutletDesc                                  ,"outlet.desc");
  (OUTLET_FREQUENCY                                 ,OutletFrequency                             ,"outlet.frequency");
  (OUTLET_GROUP_COUNT                               ,OutletGroupCount                            ,"outlet.group.count");
  (OUTLET_ID                                        ,OutletId                                    ,"outlet.id");
  (OUTLET_POWER                                     ,OutletPower                                 ,"outlet.power");
  (OUTLET_POWERFACTOR                               ,OutletPowerfactor                           ,"outlet.powerfactor");
  (OUTLET_REALPOWER                                 ,OutletRealpower                             ,"outlet.realpower");
  (OUTLET_SWITCHABLE                                ,OutletSwitchable                            ,"outlet.switchable");
  (OUTLET_VOLTAGE                                   ,OutletVoltage                               ,"outlet.voltage");
  (OUTPUT_CURRENT                                   ,OutputCurrent                               ,"output.current");
  (OUTPUT_CURRENT_MAXIMUM                           ,OutputCurrentMaximum                        ,"output.current.maximum");
  (OUTPUT_CURRENT_NOMINAL                           ,OutputCurrentNominal                        ,"output.current.nominal");
  (OUTPUT_FREQUENCY                                 ,OutputFrequency                             ,"output.frequency");
  (OUTPUT_FREQUENCY_MAXIMUM                         ,OutputFrequencyMaximum                      ,"output.frequency.maximum");
  (OUTPUT_FREQUENCY_MINIMUM                         ,OutputFrequencyMinimum                      ,"output.frequency.minimum");
  (OUTPUT_FREQUENCY_NOMINAL                         ,OutputFrequencyNominal                      ,"output.frequency.nominal");
  (OUTPUT_PHASES                                    ,OutputPhases                                ,"output.phases");
  (OUTPUT_POWER                                     ,OutputPower                                 ,"output.power");
  (OUTPUT_POWERFACTOR                               ,OutputPowerfactor                           ,"output.powerfactor");
  (OUTPUT_POWER_MAXIMUM_PERCENT                     ,OutputPowerMaximumPercent                   ,"output.power.maximum.percent");
  (OUTPUT_POWER_MINIMUM_PERCENT                     ,OutputPowerMinimumPercent                   ,"output.power.minimum.percent");
  (OUTPUT_POWER_NOMINAL                             ,OutputPowerNominal                          ,"output.power.nominal");
  (OUTPUT_POWER_PERCENT                             ,OutputPowerPercent                          ,"output.power.percent");
  (OUTPUT_REALPOWER                                 ,OutputRealpower                             ,"output.realpower");
  (OUTPUT_REALPOWER_NOMINAL                         ,OutputRealpowerNominal                      ,"output.realpower.nominal");
  (OUTPUT_VOLTAGE                                   ,OutputVoltage                               ,"output.voltage");
  (OUTPUT_VOLTAGE_HIGH                              ,OutputVoltageHigh                           ,"output.voltage.high");
  (OUTPUT_VOLTAGE_LOW                               ,OutputVoltageLow                            ,"output.voltage.low");
  (OUTPUT_VOLTAGE_MAXIMUM                           ,OutputVoltageMaximum                        ,"output.voltage.maximum");
  (OUTPUT_VOLTAGE_MINIMUM                           ,OutputVoltageMinimum                        ,"output.voltage.minimum");
  (OUTPUT_VOLTAGE_NOMINAL                           ,OutputVoltageNominal                        ,"output.voltage.nominal");
  (OUTPUT_VOLTAGE_TARGET_BATTERY                    ,OutputVoltageTargetBattery                  ,"output.voltage.target.battery");
  (OUTPUT_VOLTAGE_TARGET_LINE                       ,OutputVoltageTargetLine                     ,"output.voltage.target.line");
  (UPS_ALARM                                        ,UpsAlarm                                    ,"ups.alarm");
  (UPS_BEEPER_ENABLE                                ,UpsBeeperEnable                             ,"ups.beeper.enable");
  (UPS_BEEPER_STATUS                                ,UpsBeeperStatus                             ,"ups.beeper.status");
  (UPS_CONTACTS                                     ,UpsContacts                                 ,"ups.contacts");
  (UPS_DATE                                         ,UpsDate                                     ,"ups.date");
  (UPS_DELAY_REBOOT                                 ,UpsDelayReboot                              ,"ups.delay.reboot");
  (UPS_DELAY_RESTART                                ,UpsDelayRestart                             ,"ups.delay.restart");
  (UPS_DELAY_SHUTDOWN                               ,UpsDelayShutdown                            ,"ups.delay.shutdown");
  (UPS_DELAY_START                                  ,UpsDelayStart                               ,"ups.delay.start");
  (UPS_DESCRIPTION                                  ,UpsDescription                              ,"ups.description");
  (UPS_DEVICECHEMISTRY                              ,UpsDevicechemistry                          ,"ups.devicechemistry");
  (UPS_EFFICIENCY                                   ,UpsEfficiency                               ,"ups.efficiency");
  (UPS_FIRMWARE                                     ,UpsFirmware                                 ,"ups.firmware");
  (UPS_FIRMWARE_AUX                                 ,UpsFirmwareAux                              ,"ups.firmware.aux");
  (UPS_ID                                           ,UpsId                                       ,"ups.id");
  (UPS_INPUT_FREQUENCY                              ,UpsInputFrequency                           ,"ups.input.frequency");
  (UPS_INPUT_VOLTAGE                                ,UpsInputVoltage                             ,"ups.input.voltage");
  (UPS_LOAD                                         ,UpsLoad                                     ,"ups.load");
  (UPS_LOAD_HIGH                                    ,UpsLoadHigh                                 ,"ups.load.high");
  (UPS_LOAD_NOMINAL                                 ,UpsLoadNominal                              ,"ups.load.nominal");
  (UPS_MACADDR                                      ,UpsMacaddr                                  ,"ups.macaddr");
  (UPS_MFG                                          ,UpsMfg                                      ,"ups.mfg");
  (UPS_MFR                                          ,UpsMfr                                      ,"ups.mfr");
  (UPS_MFR_DATE                                     ,UpsMfrDate                                  ,"ups.mfr.date");
  (UPS_MODEL                                        ,UpsModel                                    ,"ups.model");
  (UPS_MODEL_AUX                                    ,UpsModelAux                                 ,"ups.model.aux");
  (UPS_MODEL_TYPE                                   ,UpsModelType                                ,"ups.model.type");
  (UPS_OUTPUT_PERCENTLOAD                           ,UpsOutputPercentload                        ,"ups.output.percentload");
  (UPS_OUTPUT_VOLTAGE                               ,UpsOutputVoltage                            ,"ups.output.voltage");
  (UPS_POWER                                        ,UpsPower                                    ,"ups.power");
  (UPS_POWER_NOMINAL                                ,UpsPowerNominal                             ,"ups.power.nominal");
  (UPS_PRODUCT                                      ,UpsProduct                                  ,"ups.product");
  (UPS_PRODUCTID                                    ,UpsProductid                                ,"ups.productid");
  (UPS_REALPOWER                                    ,UpsRealpower                                ,"ups.realpower");
  (UPS_REALPOWER_NOMINAL                            ,UpsRealpowerNominal                         ,"ups.realpower.nominal");
  (UPS_RUNTIME                                      ,UpsRuntime                                  ,"ups.runtime");
  (UPS_SERIAL                                       ,UpsSerial                                   ,"ups.serial");
  (UPS_SERIAL_INTERNAL                              ,UpsSerialInternal                           ,"ups.serial.internal");
  (UPS_SHUTDOWN                                     ,UpsShutdown                                 ,"ups.shutdown");
  (UPS_START_AUTO                                   ,UpsStartAuto                                ,"ups.start.auto");
  (UPS_START_BATTERY                                ,UpsStartBattery                             ,"ups.start.battery");
  (UPS_START_REBOOT                                 ,UpsStartReboot                              ,"ups.start.reboot");
  (UPS_STATUS                                       ,UpsStatus                                   ,"ups.status");
  (UPS_TEMPERATURE                                  ,UpsTemperature                              ,"ups.temperature");
  (UPS_TEMPERATURE_HIGH                             ,UpsTemperatureHigh                          ,"ups.temperature.high");
  (UPS_TEMPERATURE_LOW                              ,UpsTemperatureLow                           ,"ups.temperature.low");
  (UPS_TEST_DATE                                    ,UpsTestDate                                 ,"ups.test.date");
  (UPS_TEST_INTERVAL                                ,UpsTestInterval                             ,"ups.test.interval");
  (UPS_TEST_RESULT                                  ,UpsTestResult                               ,"ups.test.result");
  (UPS_TIME                                         ,UpsTime                                     ,"ups.time");
  (UPS_TIMER_REBOOT                                 ,UpsTimerReboot                              ,"ups.timer.reboot");
  (UPS_TIMER_RESTART                                ,UpsTimerRestart                             ,"ups.timer.restart");
  (UPS_TIMER_SHUTDOWN                               ,UpsTimerShutdown                            ,"ups.timer.shutdown");
  (UPS_TIMER_START                                  ,UpsTimerStart                               ,"ups.timer.start");
  (UPS_TYPE                                         ,UpsType                                     ,"ups.type");
  (UPS_VENDOR                                       ,UpsVendor                                   ,"ups.vendor");
  (UPS_VENDORID                                     ,UpsVendorid                                 ,"ups.vendorid");
  (UPS_VOLTAGE_NOMINAL                              ,UpsVoltageNominal                           ,"ups.voltage.nominal");
);

#[inline]
fn is_var_name(name: &str) -> Result<(), VarNameParseError> {
  let name = name.as_bytes();

  if name.is_empty() {
    return Err(VarNameParseError::Empty);
  }

  if let Some(first) = name.first()
    && !first.is_ascii_alphabetic()
  {
    return Err(VarNameParseError::InvalidName);
  }

  for byte in name.iter() {
    if !byte.is_ascii_rfc9271() {
      return Err(VarNameParseError::InvalidName);
    }
  }

  Ok(())
}

/// UPS variable name.
#[derive(Debug, Clone, Eq, Hash)]
pub struct VarName {
  name: Repr<StandardName, Box<str>>,
}

impl VarName {
  pub fn new<T>(name: T) -> Result<Self, VarNameParseError>
  where
    T: AsRef<str>,
  {
    let name_str: &str = name.as_ref();

    if let Ok(name) = StandardName::try_from(name_str) {
      Ok(Self {
        name: Repr::Standard(name),
      })
    } else {
      is_var_name(name_str)?;

      Ok(Self {
        name: Repr::Custom(Box::from(name_str)),
      })
    }
  }

  pub fn new_unchecked<T>(name: T) -> Self
  where
    T: AsRef<str>,
  {
    let name_str: &str = name.as_ref();

    if let Ok(name) = StandardName::try_from(name_str) {
      Self {
        name: Repr::Standard(name),
      }
    } else {
      Self {
        name: Repr::Custom(Box::from(name_str)),
      }
    }
  }

  #[inline]
  pub fn into_box_str(self) -> Box<str> {
    match self.name {
      Repr::Standard(_) => Box::from(self.as_str()),
      Repr::Custom(inner) => inner,
    }
  }

  pub fn is_valid_name(name: &str) -> bool {
    is_var_name(name).is_ok()
  }

  #[inline]
  pub const fn as_str(&self) -> &str {
    match &self.name {
      Repr::Standard(name) => name.as_str(),
      Repr::Custom(boxed_name) => boxed_name,
    }
  }
}

impl AsRef<str> for VarName {
  #[inline]
  fn as_ref(&self) -> &str {
    match &self.name {
      Repr::Standard(name) => name.as_str(),
      Repr::Custom(boxed_name) => boxed_name,
    }
  }
}

impl std::fmt::Display for VarName {
  #[inline]
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self.name {
      Repr::Standard(name) => f.write_str(name.as_str()),
      Repr::Custom(boxed_name) => f.write_str(boxed_name),
    }
  }
}

impl core::str::FromStr for VarName {
  type Err = VarNameParseError;

  #[inline]
  fn from_str(value: &str) -> Result<Self, Self::Err> {
    Self::new(value)
  }
}

impl TryFrom<Box<str>> for VarName {
  type Error = VarNameParseError;

  #[inline]
  fn try_from(value: Box<str>) -> Result<Self, Self::Error> {
    if let Ok(name) = StandardName::try_from(value.as_ref()) {
      Ok(Self {
        name: Repr::Standard(name),
      })
    } else {
      is_var_name(&value)?;

      Ok(Self {
        name: Repr::Custom(value),
      })
    }
  }
}

impl TryFrom<std::borrow::Cow<'_, str>> for VarName {
  type Error = VarNameParseError;

  #[inline]
  fn try_from(value: std::borrow::Cow<'_, str>) -> Result<Self, Self::Error> {
    match value {
      std::borrow::Cow::Borrowed(v) => Self::new(v),
      std::borrow::Cow::Owned(v) => Self::try_from(v),
    }
  }
}

impl TryFrom<String> for VarName {
  type Error = VarNameParseError;

  #[inline]
  fn try_from(value: String) -> Result<Self, Self::Error> {
    if let Ok(name) = StandardName::try_from(value.as_ref()) {
      Ok(Self {
        name: Repr::Standard(name),
      })
    } else {
      is_var_name(&value)?;

      Ok(Self {
        name: Repr::Custom(value.into_boxed_str()),
      })
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

impl PartialEq<&str> for VarName {
  #[inline]
  fn eq(&self, other: &&str) -> bool {
    *self == **other
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

impl Borrow<str> for VarName {
  #[inline]
  fn borrow(&self) -> &str {
    self.as_str()
  }
}

impl PartialOrd for VarName {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for VarName {
  #[inline]
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.as_str().cmp(other.as_str())
  }
}

impl From<VarName> for Box<str> {
  #[inline]
  fn from(value: VarName) -> Self {
    value.into_box_str()
  }
}

#[cfg(feature = "serde")]
mod serde {
  use super::VarName;
  use crate::internal::Repr;
  use serde::de::Visitor;

  impl serde::Serialize for VarName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: serde::Serializer,
    {
      match &self.name {
        Repr::Standard(name) => serializer.serialize_str(name.as_str()),
        Repr::Custom(name) => serializer.serialize_str(name),
      }
    }
  }

  struct VarNameVisitor;

  impl<'de> serde::Deserialize<'de> for VarName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: serde::Deserializer<'de>,
    {
      deserializer.deserialize_string(VarNameVisitor)
    }
  }

  impl<'de> Visitor<'de> for VarNameVisitor {
    type Value = VarName;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("expecting a variable name string")
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      VarName::new(v).map_err(|err| E::custom(err.to_string()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      VarName::try_from(v).map_err(|err| E::custom(err.to_string()))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      VarName::new(v).map_err(|err| E::custom(err.to_string()))
    }
  }
}
