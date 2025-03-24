use super::semantic_classes::{TEXT_ERROR, TEXT_INFO, TEXT_SUCCESS, TEXT_WARNING};
use std::str::SplitAsciiWhitespace;

#[derive(Debug)]
pub struct StatusIcon {
  pub name: &'static str,
  pub class: &'static str,
  pub desc: &'static str,
}

pub struct StatusIconIter<'a> {
  inner: SplitAsciiWhitespace<'a>,
}

impl<'a> StatusIconIter<'a> {
  pub fn new(status_text: &'a str) -> Self {
    Self {
      inner: status_text.split_ascii_whitespace(),
    }
  }
}

impl<'a> Iterator for StatusIconIter<'a> {
  type Item = StatusIcon;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let token = self.inner.next()?;

      let icon = match token {
        "OL" => Some(StatusIcon {
          name: "activity",
          class: TEXT_SUCCESS,
          desc: "UPS Online",
        }),
        "BOOST" => Some(StatusIcon {
          name: "zap-off",
          class: TEXT_ERROR,
          desc: "Input voltage is too low",
        }),
        "CAL" => Some(StatusIcon {
          name: "loader",
          class: TEXT_INFO,
          desc: "UPS is calibrating",
        }),
        "CHRG" => Some(StatusIcon {
          name: "battery-charging",
          class: TEXT_SUCCESS,
          desc: "Battery is charging",
        }),
        "DISCHRG" => Some(StatusIcon {
          name: "battery",
          class: TEXT_INFO,
          desc: "Battery is discharging",
        }),
        "ALARM" => Some(StatusIcon {
          name: "alert-circle",
          class: TEXT_ERROR,
          desc: "UPS requires intervention",
        }),
        "FSD" => Some(StatusIcon {
          name: "power",
          class: TEXT_ERROR,
          desc: "FORCED SHUTDOWN has begun",
        }),
        "LB" => Some(StatusIcon {
          name: "battery",
          class: TEXT_WARNING,
          desc: "Low battery",
        }),
        "OB" => Some(StatusIcon {
          name: "battery-charging",
          class: TEXT_WARNING,
          desc: "UPS on battery",
        }),
        "OVER" => Some(StatusIcon {
          name: "alert-circle",
          class: TEXT_ERROR,
          desc: "UPS is overloaded",
        }),
        "NOCOMM" => Some(StatusIcon {
          name: "wifi-off",
          class: TEXT_ERROR,
          desc: "Daemon has no connection with UPS",
        }),
        "RB" => Some(StatusIcon {
          name: "tool",
          class: TEXT_ERROR,
          desc: "Replace battery",
        }),
        "TEST" => Some(StatusIcon {
          name: "activity",
          class: TEXT_INFO,
          desc: "Under test",
        }),
        "TRIM" => Some(StatusIcon {
          name: "zap",
          class: TEXT_ERROR,
          desc: "Input voltage is too high",
        }),
        "BYPASS" => Some(StatusIcon {
          name: "tool",
          class: TEXT_INFO,
          desc: "Battery is disconnected for maintenance",
        }),
        _ => continue,
      };

      return icon;
    }
  }
}

pub fn get_status_text(value: &str) -> &str {
  match value {
    "OL" => "ONLINE",
    "BOOST" => "LOW VOLTAGE",
    "CAL" => "CALIBRATING",
    "CHRG" => "CHARGING",
    "DISCHRG" => "DISCHARGING",
    "FSD" => "FORCED SHUTDOWN",
    "LB" => "LOW BATTERY",
    "OB" => "ON BATTERY",
    "OFF" => "OFFLINE",
    "OVER" => "OVERLOADED",
    "RB" => "REPLACE BATTERY",
    "TEST" => "TESTING",
    "TRIM" => "HIGH VOLTAGE",
    val => val,
  }
}

pub fn get_status_class(value: &str) -> Option<&'static str> {
  match value {
    "OL" => Some(TEXT_SUCCESS),
    "BOOST" => Some(TEXT_ERROR),
    "CAL" => Some(TEXT_INFO),
    "CHRG" => Some(TEXT_INFO),
    "DISCHRG" => Some(TEXT_WARNING),
    "ALARM" => Some(TEXT_ERROR),
    "FSD" => Some(TEXT_ERROR),
    "LB" => Some(TEXT_ERROR),
    "OB" => Some(TEXT_WARNING),
    "OVER" => Some(TEXT_ERROR),
    "NOCOMM" => Some(TEXT_ERROR),
    "RB" => Some(TEXT_ERROR),
    "TEST" => Some(TEXT_INFO),
    "TRIM" => Some(TEXT_ERROR),
    "BYPASS" => Some(TEXT_WARNING),
    _ => None,
  }
}
