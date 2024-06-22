use crate::ups_mem_store::UpsEntry;
use crate::upsd_client::ups_variables::UpsVariable;
use std::borrow::Borrow;

#[derive(Debug)]
pub struct UpsInfo {
  pub name: String,
  pub desc: String,
  pub load: Option<u8>,
  pub power_nominal: Option<f64>,
  pub power: Option<f64>,
  pub status: Option<String>,
  pub runtime: Option<i32>,
  pub charge: Option<u8>,
  pub charge_low: Option<u8>,
  pub battery_voltage: Option<f64>,
  pub input_voltage: Option<f64>,
  pub beeper_status: Option<bool>,
}

impl UpsInfo {
  pub fn from_ups_entry(ups: &UpsEntry) -> UpsInfo {
    let mut ups_info = UpsInfo {
      name: String::from(ups.name.as_ref()),
      desc: String::from(ups.desc.as_ref()),
      status: None,
      load: None,
      runtime: None,
      charge_low: None,
      charge: None,
      battery_voltage: None,
      input_voltage: None,
      power_nominal: None,
      power: None,
      beeper_status: None,
    };

    for variable in &ups.variables {
      match variable {
        UpsVariable::UpsLoad(val) => {
          ups_info.load = Some(*val);
        }
        UpsVariable::UpsPowerNominal(val) => {
          ups_info.power_nominal = Some(*val);
        }
        UpsVariable::UpsPower(val) => {
          ups_info.power = Some(*val);
        }
        UpsVariable::BatteryCharge(val) => {
          ups_info.charge = Some(*val);
        }
        UpsVariable::BatteryLow(val) => {
          ups_info.charge_low = Some(*val);
        }
        UpsVariable::BatteryRuntime(val) => {
          ups_info.runtime = Some(*val);
        }
        UpsVariable::UpsStatus(val) => {
          ups_info.status = Some(val.to_string());
        }
        UpsVariable::BatteryVoltage(val) => {
          ups_info.battery_voltage = Some(*val);
        }
        UpsVariable::InputVoltage(val) => {
          ups_info.input_voltage = Some(*val);
        }
        UpsVariable::UpsBeeperStatus(val) => {
          ups_info.beeper_status = match val.as_str() {
            "enabled" => Some(true),
            _ => Some(false),
          };
        }
        _ => {}
      }
    }

    if let UpsInfo {
      power_nominal: Some(pw),
      load: Some(ld),
      power: None,
      ..
    } = ups_info
    {
      ups_info.power = Some((pw * f64::from(ld)) / 100.0_f64);
    };

    ups_info
  }
}

impl<T> From<T> for UpsInfo
where
  T: Borrow<UpsEntry>,
{
  fn from(value: T) -> Self {
    Self::from_ups_entry(value.borrow())
  }
}
