use nut_webgui_upsmc::VarName;
use prometheus_client::{metrics::MetricType, registry::Unit};

pub const METRIC_UPS_STATUS: &str = "ups_status";
pub const METRIC_UPS_STATUS_HELP: &str = "UPS status";

pub struct MetricDescriptor<'a> {
  pub help: &'a str,
  pub name: &'a str,
  pub unit: Option<Unit>,
  pub metric_type: MetricType,
  pub target_name: VarName,
}

#[derive(Clone, Copy)]
pub struct KnownMetricDescriptors;

impl KnownMetricDescriptors {
  pub fn from_var_name(name: &VarName) -> Option<&'static MetricDescriptor<'static>> {
    match KNOWN_DESCRIPTORS.binary_search_by(|v| v.target_name.cmp(name)) {
      Ok(idx) => Some(&KNOWN_DESCRIPTORS[idx]),
      Err(_) => None,
    }
  }
}

static KNOWN_DESCRIPTORS: &[MetricDescriptor<'static>] = &[
  MetricDescriptor {
    help: "Ambient humidity",
    name: "ambient_humidity",
    unit: None,
    metric_type: MetricType::Gauge,
    target_name: VarName::AMBIENT_HUMIDITY,
  },
  MetricDescriptor {
    help: "Ambient temperature",
    name: "ambient_temperature",
    unit: Some(Unit::Celsius),
    metric_type: MetricType::Gauge,
    target_name: VarName::AMBIENT_TEMPERATURE,
  },
  MetricDescriptor {
    help: "Battery charge level",
    name: "battery_charge",
    unit: None,
    metric_type: MetricType::Gauge,
    target_name: VarName::BATTERY_CHARGE,
  },
  MetricDescriptor {
    help: "Battery current",
    name: "battery_current",
    unit: Some(Unit::Amperes),
    metric_type: MetricType::Gauge,
    target_name: VarName::BATTERY_CURRENT,
  },
  MetricDescriptor {
    help: "Estimated battery runtime",
    name: "battery_runtime",
    unit: Some(Unit::Seconds),
    metric_type: MetricType::Gauge,
    target_name: VarName::BATTERY_RUNTIME,
  },
  MetricDescriptor {
    help: "Battery temperature",
    name: "battery_temperature",
    unit: Some(Unit::Celsius),
    metric_type: MetricType::Gauge,
    target_name: VarName::BATTERY_TEMPERATURE,
  },
  MetricDescriptor {
    help: "Battery voltage",
    name: "battery_voltage",
    unit: Some(Unit::Volts),
    metric_type: MetricType::Gauge,
    target_name: VarName::BATTERY_VOLTAGE,
  },
  MetricDescriptor {
    help: "Input bypass current",
    name: "input_bypass_current",
    unit: Some(Unit::Amperes),
    metric_type: MetricType::Gauge,
    target_name: VarName::INPUT_BYPASS_CURRENT,
  },
  MetricDescriptor {
    help: "Input bypass frequency",
    name: "input_bypass_frequency",
    unit: None,
    metric_type: MetricType::Gauge,
    target_name: VarName::INPUT_BYPASS_FREQUENCY,
  },
  MetricDescriptor {
    help: "Input bypass voltage",
    name: "input_bypass_voltage",
    unit: Some(Unit::Volts),
    metric_type: MetricType::Gauge,
    target_name: VarName::INPUT_BYPASS_VOLTAGE,
  },
  MetricDescriptor {
    help: "Number of input sources",
    name: "input_count",
    unit: None,
    metric_type: MetricType::Gauge,
    target_name: VarName::INPUT_COUNT,
  },
  MetricDescriptor {
    help: "Input current",
    name: "input_current",
    unit: Some(Unit::Amperes),
    metric_type: MetricType::Gauge,
    target_name: VarName::INPUT_CURRENT,
  },
  MetricDescriptor {
    help: "Input frequency",
    name: "input_frequency",
    unit: None,
    metric_type: MetricType::Gauge,
    target_name: VarName::INPUT_FREQUENCY,
  },
  MetricDescriptor {
    help: "Input load percentage",
    name: "input_load",
    unit: None,
    metric_type: MetricType::Gauge,
    target_name: VarName::INPUT_LOAD,
  },
  MetricDescriptor {
    help: "Input power in VA",
    name: "input_power",
    unit: None,
    metric_type: MetricType::Gauge,
    target_name: VarName::INPUT_POWER,
  },
  MetricDescriptor {
    help: "Input real power in Watt",
    name: "input_realpower",
    unit: None,
    metric_type: MetricType::Gauge,
    target_name: VarName::INPUT_REALPOWER,
  },
  MetricDescriptor {
    help: "Input voltage",
    name: "input_voltage",
    unit: Some(Unit::Volts),
    metric_type: MetricType::Gauge,
    target_name: VarName::INPUT_VOLTAGE,
  },
  MetricDescriptor {
    help: "Number of outlets",
    name: "outlet_count",
    unit: None,
    metric_type: MetricType::Gauge,
    target_name: VarName::OUTLET_COUNT,
  },
  MetricDescriptor {
    help: "Outlet current",
    name: "outlet_current",
    unit: Some(Unit::Amperes),
    metric_type: MetricType::Gauge,
    target_name: VarName::OUTLET_CURRENT,
  },
  MetricDescriptor {
    help: "Outlet power in VA",
    name: "outlet_power",
    unit: None,
    metric_type: MetricType::Gauge,
    target_name: VarName::OUTLET_POWER,
  },
  MetricDescriptor {
    help: "Outlet real power in Watt",
    name: "outlet_realpower",
    unit: None,
    metric_type: MetricType::Gauge,
    target_name: VarName::OUTLET_REALPOWER,
  },
  MetricDescriptor {
    help: "Outlet voltage",
    name: "outlet_voltage",
    unit: Some(Unit::Volts),
    metric_type: MetricType::Gauge,
    target_name: VarName::OUTLET_VOLTAGE,
  },
  MetricDescriptor {
    help: "Output current",
    name: "output_current",
    unit: Some(Unit::Amperes),
    metric_type: MetricType::Gauge,
    target_name: VarName::OUTPUT_CURRENT,
  },
  MetricDescriptor {
    help: "Output frequency",
    name: "output_frequency",
    unit: None,
    metric_type: MetricType::Gauge,
    target_name: VarName::OUTPUT_FREQUENCY,
  },
  MetricDescriptor {
    help: "Output power in VA",
    name: "output_power",
    unit: None,
    metric_type: MetricType::Gauge,
    target_name: VarName::OUTPUT_POWER,
  },
  MetricDescriptor {
    help: "Output power percentage",
    name: "output_power_percent",
    unit: None,
    metric_type: MetricType::Gauge,
    target_name: VarName::OUTPUT_POWER_PERCENT,
  },
  MetricDescriptor {
    help: "Output real power in Watt",
    name: "output_realpower",
    unit: None,
    metric_type: MetricType::Gauge,
    target_name: VarName::OUTPUT_REALPOWER,
  },
  MetricDescriptor {
    help: "Output voltage",
    name: "output_voltage",
    unit: Some(Unit::Volts),
    metric_type: MetricType::Gauge,
    target_name: VarName::OUTPUT_VOLTAGE,
  },
  MetricDescriptor {
    help: "Beeper enable status",
    name: "ups_beeper_enable",
    unit: None,
    metric_type: MetricType::Gauge,
    target_name: VarName::UPS_BEEPER_ENABLE,
  },
  MetricDescriptor {
    help: "Beeper status",
    name: "ups_beeper_status",
    unit: None,
    metric_type: MetricType::Gauge,
    target_name: VarName::UPS_BEEPER_STATUS,
  },
  MetricDescriptor {
    help: "UPS input frequency",
    name: "ups_input_frequency",
    unit: None,
    metric_type: MetricType::Gauge,
    target_name: VarName::UPS_INPUT_FREQUENCY,
  },
  MetricDescriptor {
    help: "UPS input voltage",
    name: "ups_input_voltage",
    unit: Some(Unit::Volts),
    metric_type: MetricType::Gauge,
    target_name: VarName::UPS_INPUT_VOLTAGE,
  },
  MetricDescriptor {
    help: "UPS load percentage",
    name: "ups_load",
    unit: None,
    metric_type: MetricType::Gauge,
    target_name: VarName::UPS_LOAD,
  },
  MetricDescriptor {
    help: "UPS output voltage",
    name: "ups_output_voltage",
    unit: Some(Unit::Volts),
    metric_type: MetricType::Gauge,
    target_name: VarName::UPS_OUTPUT_VOLTAGE,
  },
  MetricDescriptor {
    help: "UPS power in VA",
    name: "ups_power",
    unit: None,
    metric_type: MetricType::Gauge,
    target_name: VarName::UPS_POWER,
  },
  MetricDescriptor {
    help: "UPS real power in Watt",
    name: "ups_realpower",
    unit: None,
    metric_type: MetricType::Gauge,
    target_name: VarName::UPS_REALPOWER,
  },
  MetricDescriptor {
    help: "UPS estimated runtime",
    name: "ups_runtime",
    unit: Some(Unit::Seconds),
    metric_type: MetricType::Gauge,
    target_name: VarName::UPS_RUNTIME,
  },
  MetricDescriptor {
    help: "UPS temperature",
    name: "ups_temperature",
    unit: Some(Unit::Celsius),
    metric_type: MetricType::Gauge,
    target_name: VarName::UPS_TEMPERATURE,
  },
];

#[cfg(test)]
mod test {
  use super::KNOWN_DESCRIPTORS;

  #[test]
  fn is_sorted() {
    assert!(KNOWN_DESCRIPTORS.is_sorted_by(|a, b| a.target_name <= b.target_name))
  }
}
