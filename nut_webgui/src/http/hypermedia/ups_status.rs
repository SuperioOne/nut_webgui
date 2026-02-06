use super::semantic_type::SemanticType;
use nut_webgui_upsmc::ups_status::UpsStatus;

#[derive(Debug)]
pub struct StatusDetail {
  pub icon_name: &'static str,
  pub class: SemanticType,
  pub desc: &'static str,
  pub name: &'static str,
}

pub struct StatusDetailIter {
  inner: nut_webgui_upsmc::ups_status::Iter,
}

impl<'a> StatusDetailIter {
  #[inline]
  pub const fn new(status: UpsStatus) -> Self {
    Self {
      inner: status.iter(),
    }
  }
}

impl<'a> Iterator for StatusDetailIter {
  type Item = StatusDetail;

  fn next(&mut self) -> Option<Self::Item> {
    let status = self.inner.next()?;
    get_status_detail(status)
  }
}

fn get_status_detail(status: UpsStatus) -> Option<StatusDetail> {
  match status {
    UpsStatus::ONLINE => Some(StatusDetail {
      icon_name: "activity",
      class: SemanticType::Success,
      desc: "UPS Online",
      name: "ONLINE",
    }),
    UpsStatus::BOOST => Some(StatusDetail {
      icon_name: "zap-off",
      class: SemanticType::Error,
      desc: "Input voltage is too low",
      name: "BOOST",
    }),
    UpsStatus::CALIBRATING => Some(StatusDetail {
      icon_name: "loader",
      class: SemanticType::Info,
      desc: "UPS is calibrating",
      name: "CALIBRATING",
    }),
    UpsStatus::CHARGING => Some(StatusDetail {
      icon_name: "battery-charging",
      class: SemanticType::Info,
      desc: "Battery is charging",
      name: "CHARGING",
    }),
    UpsStatus::DISCHARGE => Some(StatusDetail {
      icon_name: "battery",
      class: SemanticType::Warning,
      desc: "Battery is discharging",
      name: "DISCHARGING",
    }),
    UpsStatus::ALARM => Some(StatusDetail {
      icon_name: "alert-circle",
      class: SemanticType::Error,
      desc: "UPS requires intervention",
      name: "ALARM",
    }),
    UpsStatus::FORCED_SHUTDOWN => Some(StatusDetail {
      icon_name: "power",
      class: SemanticType::Error,
      desc: "FORCED SHUTDOWN has begun",
      name: "FSD",
    }),
    UpsStatus::LOW_BATTERY => Some(StatusDetail {
      icon_name: "battery",
      class: SemanticType::Warning,
      desc: "Low battery",
      name: "LOW-BATTERY",
    }),
    UpsStatus::ON_BATTERY => Some(StatusDetail {
      icon_name: "battery-charging",
      class: SemanticType::Warning,
      desc: "UPS on battery",
      name: "ON-BATTERY",
    }),
    UpsStatus::OVERLOADED => Some(StatusDetail {
      icon_name: "alert-circle",
      class: SemanticType::Error,
      desc: "UPS is overloaded",
      name: "OVERLOADED",
    }),
    UpsStatus::NOCOMM => Some(StatusDetail {
      icon_name: "wifi-off",
      class: SemanticType::Error,
      desc: "Daemon has no connection with UPS",
      name: "NOCOMM",
    }),
    UpsStatus::REPLACE_BATTERY => Some(StatusDetail {
      icon_name: "tool",
      class: SemanticType::Error,
      desc: "Replace battery",
      name: "REPLACE-BATTERY",
    }),
    UpsStatus::TEST => Some(StatusDetail {
      icon_name: "activity",
      class: SemanticType::Info,
      desc: "Under test",
      name: "TEST",
    }),
    UpsStatus::TRIM => Some(StatusDetail {
      icon_name: "zap",
      class: SemanticType::Error,
      desc: "Input voltage is too high",
      name: "TRIM",
    }),
    UpsStatus::BYPASS => Some(StatusDetail {
      icon_name: "tool",
      class: SemanticType::Info,
      desc: "Battery is disconnected for maintenance",
      name: "BYPASS",
    }),
    _ => None,
  }
}
