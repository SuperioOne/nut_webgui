use super::semantic_type::SemanticType;
use std::str::SplitAsciiWhitespace;

#[derive(Debug)]
pub struct StatusDetail<'a> {
  pub icon_name: &'a str,
  pub class: SemanticType,
  pub desc: &'a str,
  pub name: &'a str,
}

pub struct StatusDetailIter<'a> {
  inner: SplitAsciiWhitespace<'a>,
}

impl<'a> StatusDetailIter<'a> {
  pub fn new(status_text: &'a str) -> Self {
    Self {
      inner: status_text.split_ascii_whitespace(),
    }
  }
}

impl<'a> Iterator for StatusDetailIter<'a> {
  type Item = StatusDetail<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    let token = self.inner.next()?;
    Some(str_to_status(token))
  }
}

fn str_to_status(token: &str) -> StatusDetail<'_> {
  match token {
    "OL" => StatusDetail {
      icon_name: "activity",
      class: SemanticType::Success,
      desc: "UPS Online",
      name: "ONLINE",
    },
    "BOOST" => StatusDetail {
      icon_name: "zap-off",
      class: SemanticType::Error,
      desc: "Input voltage is too low",
      name: token,
    },
    "CAL" => StatusDetail {
      icon_name: "loader",
      class: SemanticType::Info,
      desc: "UPS is calibrating",
      name: "CALIBRATING",
    },
    "CHRG" => StatusDetail {
      icon_name: "battery-charging",
      class: SemanticType::Info,
      desc: "Battery is charging",
      name: "CHARGING",
    },
    "DISCHRG" => StatusDetail {
      icon_name: "battery",
      class: SemanticType::Warning,
      desc: "Battery is discharging",
      name: "DISCHARGING",
    },
    "ALARM" => StatusDetail {
      icon_name: "alert-circle",
      class: SemanticType::Error,
      desc: "UPS requires intervention",
      name: token,
    },
    "FSD" => StatusDetail {
      icon_name: "power",
      class: SemanticType::Error,
      desc: "FORCED SHUTDOWN has begun",
      name: token,
    },
    "LB" => StatusDetail {
      icon_name: "battery",
      class: SemanticType::Warning,
      desc: "Low battery",
      name: "LOW-BATTERY",
    },
    "OB" => StatusDetail {
      icon_name: "battery-charging",
      class: SemanticType::Warning,
      desc: "UPS on battery",
      name: "ON-BATTERY",
    },
    "OVER" => StatusDetail {
      icon_name: "alert-circle",
      class: SemanticType::Error,
      desc: "UPS is overloaded",
      name: "OVERLOADED",
    },
    "NOCOMM" => StatusDetail {
      icon_name: "wifi-off",
      class: SemanticType::Error,
      desc: "Daemon has no connection with UPS",
      name: token,
    },
    "RB" => StatusDetail {
      icon_name: "tool",
      class: SemanticType::Error,
      desc: "Replace battery",
      name: "REPLACE-BATTERY",
    },
    "TEST" => StatusDetail {
      icon_name: "activity",
      class: SemanticType::Info,
      desc: "Under test",
      name: token,
    },
    "TRIM" => StatusDetail {
      icon_name: "zap",
      class: SemanticType::Error,
      desc: "Input voltage is too high",
      name: token,
    },
    "BYPASS" => StatusDetail {
      icon_name: "tool",
      class: SemanticType::Info,
      desc: "Battery is disconnected for maintenance",
      name: token,
    },
    name => StatusDetail {
      icon_name: "",
      class: SemanticType::Info,
      desc: "Description not available",
      name,
    },
  }
}
