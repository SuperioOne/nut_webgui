use crate::upsd_client::protocol::{UpsVariable};
use crate::ups_mem_store::UpsEntry;

#[derive(Debug)]
pub struct UpsInfo {
  pub name: String,
  pub desc: String,
  pub load: Option<u8>,
  pub power_nominal: Option<u32>,
  pub status: Option<String>,
  pub runtime: Option<i32>,
  pub charge: Option<u8>,
  pub charge_low: Option<u8>,
}

impl UpsInfo {
  pub fn load_from(ups: &UpsEntry) -> UpsInfo {
    let mut ups_info = UpsInfo {
      name: String::from(ups.name.as_ref()),
      desc: String::from(ups.desc.as_ref()),
      status: None,
      load: None,
      runtime: None,
      charge_low: None,
      charge: None,
      power_nominal: None,
    };

    for variable in &ups.variables {
      match variable {
        UpsVariable::UpsLoad(val) => { ups_info.load = Some(*val); }
        UpsVariable::UpsPowerNominal(val) => { ups_info.power_nominal = Some(*val); }
        UpsVariable::BatteryCharge(val) => { ups_info.charge = Some(*val); }
        UpsVariable::BatteryLow(val) => { ups_info.charge_low = Some(*val); }
        UpsVariable::BatteryRuntime(val) => { ups_info.runtime = Some(*val); }
        UpsVariable::UpsStatus(val) => { ups_info.status = Some(val.to_string()); }
        _ => {}
      }
    }

    ups_info
  }
}
