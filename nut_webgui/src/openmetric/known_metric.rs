use nut_webgui_upsmc::{VarName, ups_variables::UpsVariables};
use prometheus_client::{metrics::MetricType, registry::Unit};
use std::sync::LazyLock;

pub const METRIC_UPS_STATUS: &str = "ups_status";
pub const METRIC_UPS_STATUS_HELP: &str = "UPS status";
static UNIT_WATT: LazyLock<Unit> = LazyLock::new(|| Unit::Other("watts".to_owned()));
static UNIT_VA: LazyLock<Unit> = LazyLock::new(|| Unit::Other("voltamps".to_owned()));
static UNIT_HZ: LazyLock<Unit> = LazyLock::new(|| Unit::Other("hertzs".to_owned()));

pub trait MetricDescriptor: Send + Sync + 'static {
  fn help(&self) -> &str;
  fn metric_family(&self) -> &str;
  fn unit(&self) -> Option<&Unit>;
  fn metric_type(&self) -> MetricType;
  fn value(&self, variables: &UpsVariables) -> Option<f64>;
}

macro_rules! generic_descriptor {
  ($vis:vis $name:ident {
      help: $help:literal,
      name: $metric_family:literal,
      unit: $unit:expr,
      metric_type: $metric:expr,
      target_name: $target:expr
  }) => {
    #[derive(Clone, Copy)]
    $vis struct $name;

    impl MetricDescriptor for $name {
      fn help(&self) -> &str {
        $help
      }

      fn metric_family(&self) -> &str {
        $metric_family
      }

      fn unit(&self) -> Option<&Unit> {
        $unit
      }

      fn metric_type(&self) -> MetricType {
          $metric
      }

      fn value(&self, variables: &UpsVariables) -> Option<f64> {
          variables.get($target).map(|v| { v.as_lossy_f64() }).flatten()
      }
    }
  };
}

#[derive(Clone, Copy)]
pub struct UpsPower;

#[derive(Clone, Copy)]
pub struct UpsRealpower;

impl MetricDescriptor for UpsPower {
  fn help(&self) -> &str {
    "UPS power in VA"
  }

  fn metric_family(&self) -> &str {
    "ups_power"
  }

  fn unit(&self) -> Option<&Unit> {
    Some(&UNIT_VA)
  }

  fn metric_type(&self) -> MetricType {
    MetricType::Gauge
  }

  fn value(&self, variables: &UpsVariables) -> Option<f64> {
    variables
      .get(VarName::UPS_POWER)
      .map(|v| v.as_lossy_f64())
      .flatten()
      .or_else(|| {
        let load = variables
          .get(VarName::UPS_LOAD)
          .and_then(|v| v.as_lossy_f64())?;

        let nominal_power = variables
          .get(VarName::UPS_POWER_NOMINAL)
          .and_then(|v| v.as_lossy_f64())?;

        Some((nominal_power * load / 100.0).round())
      })
  }
}

impl MetricDescriptor for UpsRealpower {
  fn help(&self) -> &str {
    "UPS real power in Watt"
  }

  fn metric_family(&self) -> &str {
    "ups_realpower"
  }

  fn unit(&self) -> Option<&Unit> {
    Some(&UNIT_WATT)
  }

  fn metric_type(&self) -> MetricType {
    MetricType::Gauge
  }

  fn value(&self, variables: &UpsVariables) -> Option<f64> {
    variables
      .get(VarName::UPS_REALPOWER)
      .map(|v| v.as_lossy_f64())
      .flatten()
      .or_else(|| {
        let load = variables
          .get(VarName::UPS_LOAD)
          .and_then(|v| v.as_lossy_f64())?;

        let nominal_power = variables
          .get(VarName::UPS_REALPOWER_NOMINAL)
          .and_then(|v| v.as_lossy_f64())?;

        Some((nominal_power * load / 100.0).round())
      })
  }
}

generic_descriptor!(pub InputPower {
  help: "Input power in VA",
  name: "input_power",
  unit: Some(&UNIT_VA),
  metric_type: MetricType::Gauge,
  target_name: VarName::INPUT_POWER
});

generic_descriptor!(pub InputRealpower {
  help: "Input real power in Watt",
  name: "input_realpower",
  unit: Some(&UNIT_WATT),
  metric_type: MetricType::Gauge,
  target_name: VarName::INPUT_REALPOWER
});

generic_descriptor!(pub AmbientHumidity {
  help: "Ambient humidity",
  name: "ambient_humidity",
  unit: None,
  metric_type: MetricType::Gauge,
  target_name: VarName::AMBIENT_HUMIDITY
});

generic_descriptor!(pub AmbientTemperature {
  help: "Ambient temperature",
  name: "ambient_temperature",
  unit: Some(&Unit::Celsius),
  metric_type: MetricType::Gauge,
  target_name: VarName::AMBIENT_TEMPERATURE
});

generic_descriptor!(pub BatteryCharge {
  help: "Battery charge level",
  name: "battery_charge",
  unit: None,
  metric_type: MetricType::Gauge,
  target_name: VarName::BATTERY_CHARGE
});

generic_descriptor!(pub BatteryCurrent {
  help: "Battery current",
  name: "battery_current",
  unit: Some(&Unit::Amperes),
  metric_type: MetricType::Gauge,
  target_name: VarName::BATTERY_CURRENT
});

generic_descriptor!(pub BatteryRuntime {
  help: "Estimated battery runtime",
  name: "battery_runtime",
  unit: Some(&Unit::Seconds),
  metric_type: MetricType::Gauge,
  target_name: VarName::BATTERY_RUNTIME
});

generic_descriptor!(pub BatteryTemperature {
  help: "Battery temperature",
  name: "battery_temperature",
  unit: Some(&Unit::Celsius),
  metric_type: MetricType::Gauge,
  target_name: VarName::BATTERY_TEMPERATURE
});

generic_descriptor!(pub BatteryVoltage {
  help: "Battery voltage",
  name: "battery_voltage",
  unit: Some(&Unit::Volts),
  metric_type: MetricType::Gauge,
  target_name: VarName::BATTERY_VOLTAGE
});

generic_descriptor!(pub InputBypassCurrent {
  help: "Input bypass current",
  name: "input_bypass_current",
  unit: Some(&Unit::Amperes),
  metric_type: MetricType::Gauge,
  target_name: VarName::INPUT_BYPASS_CURRENT
});

generic_descriptor!(pub InputBypassFrequency {
  help: "Input bypass frequency",
  name: "input_bypass_frequency",
  unit: Some(&UNIT_HZ),
  metric_type: MetricType::Gauge,
  target_name: VarName::INPUT_BYPASS_FREQUENCY
});

generic_descriptor!(pub InputBypassVoltage {
  help: "Input bypass voltage",
  name: "input_bypass_voltage",
  unit: Some(&Unit::Volts),
  metric_type: MetricType::Gauge,
  target_name: VarName::INPUT_BYPASS_VOLTAGE
});

generic_descriptor!(pub InputCount {
  help: "Number of input sources",
  name: "input_count",
  unit: None,
  metric_type: MetricType::Gauge,
  target_name: VarName::INPUT_COUNT
});

generic_descriptor!(pub InputCurrent {
  help: "Input current",
  name: "input_current",
  unit: Some(&Unit::Amperes),
  metric_type: MetricType::Gauge,
  target_name: VarName::INPUT_CURRENT
});

generic_descriptor!(pub InputFrequency {
  help: "Input frequency",
  name: "input_frequency",
  unit: Some(&UNIT_HZ),
  metric_type: MetricType::Gauge,
  target_name: VarName::INPUT_FREQUENCY
});

generic_descriptor!(pub InputLoadPercentage {
  help: "Input load percentage",
  name: "input_load",
  unit: None,
  metric_type: MetricType::Gauge,
  target_name: VarName::INPUT_LOAD
});

generic_descriptor!(pub InputVoltage {
  help: "Input voltage",
  name: "input_voltage",
  unit: Some(&Unit::Volts),
  metric_type: MetricType::Gauge,
  target_name: VarName::INPUT_VOLTAGE
});

generic_descriptor!(pub OutletCount {
  help: "Number of outlets",
  name: "outlet_count",
  unit: None,
  metric_type: MetricType::Gauge,
  target_name: VarName::OUTLET_COUNT
});

generic_descriptor!(pub OutletCurrent {
  help: "Outlet current",
  name: "outlet_current",
  unit: Some(&Unit::Amperes),
  metric_type: MetricType::Gauge,
  target_name: VarName::OUTLET_CURRENT
});

generic_descriptor!(pub OutletPower {
  help: "Outlet power in VA",
  name: "outlet_power",
  unit: Some(&UNIT_VA),
  metric_type: MetricType::Gauge,
  target_name: VarName::OUTLET_POWER
});

generic_descriptor!(pub OutletRealpower {
  help: "Outlet real power in Watt",
  name: "outlet_realpower",
  unit: Some(&UNIT_WATT),
  metric_type: MetricType::Gauge,
  target_name: VarName::OUTLET_REALPOWER
});

generic_descriptor!(pub OutletVoltage {
  help: "Outlet voltage",
  name: "outlet_voltage",
  unit: Some(&Unit::Volts),
  metric_type: MetricType::Gauge,
  target_name: VarName::OUTLET_VOLTAGE
});

generic_descriptor!(pub OutputCurrent {
  help: "Output current",
  name: "output_current",
  unit: Some(&Unit::Amperes),
  metric_type: MetricType::Gauge,
  target_name: VarName::OUTPUT_CURRENT
});

generic_descriptor!(pub OutputFrequency {
  help: "Output frequency",
  name: "output_frequency",
  unit: Some(&UNIT_HZ),
  metric_type: MetricType::Gauge,
  target_name: VarName::OUTPUT_FREQUENCY
});

generic_descriptor!(pub OutputPower {
  help: "Output power in VA",
  name: "output_power",
  unit: Some(&UNIT_VA),
  metric_type: MetricType::Gauge,
  target_name: VarName::OUTPUT_POWER
});

generic_descriptor!(pub OutputPowerPercent {
  help: "Output power percentage",
  name: "output_power_percent",
  unit: None,
  metric_type: MetricType::Gauge,
  target_name: VarName::OUTPUT_POWER_PERCENT
});

generic_descriptor!(pub OutputRealpower {
  help: "Output real power in Watt",
  name: "output_realpower",
  unit: Some(&UNIT_WATT),
  metric_type: MetricType::Gauge,
  target_name: VarName::OUTPUT_REALPOWER
});

generic_descriptor!(pub OutputVoltage {
  help: "Output voltage",
  name: "output_voltage",
  unit: Some(&Unit::Volts),
  metric_type: MetricType::Gauge,
  target_name: VarName::OUTPUT_VOLTAGE
});

generic_descriptor!(pub UpsLoad {
  help: "UPS load percentage",
  name: "ups_load",
  unit: None,
  metric_type: MetricType::Gauge,
  target_name: VarName::UPS_LOAD
});

generic_descriptor!(pub UpsRuntime {
  help: "UPS estimated runtime",
  name: "ups_runtime",
  unit: Some(&Unit::Seconds),
  metric_type: MetricType::Gauge,
  target_name: VarName::UPS_RUNTIME
});

generic_descriptor!(pub UpsTemperature {
  help: "UPS temperature",
  name: "ups_temperature",
  unit: Some(&Unit::Celsius),
  metric_type: MetricType::Gauge,
  target_name: VarName::UPS_TEMPERATURE
});

pub static KNOWN_DESCRIPTORS: &[&'static dyn MetricDescriptor; 33] = &[
  &AmbientHumidity,
  &AmbientTemperature,
  &BatteryCharge,
  &BatteryCurrent,
  &BatteryRuntime,
  &BatteryTemperature,
  &BatteryVoltage,
  &InputBypassCurrent,
  &InputBypassFrequency,
  &InputBypassVoltage,
  &InputCount,
  &InputCurrent,
  &InputFrequency,
  &InputLoadPercentage,
  &InputPower,
  &InputRealpower,
  &InputVoltage,
  &OutletCount,
  &OutletCurrent,
  &OutletPower,
  &OutletRealpower,
  &OutletVoltage,
  &OutputCurrent,
  &OutputFrequency,
  &OutputPower,
  &OutputPowerPercent,
  &OutputRealpower,
  &OutputVoltage,
  &UpsLoad,
  &UpsPower,
  &UpsRealpower,
  &UpsRuntime,
  &UpsTemperature,
];
