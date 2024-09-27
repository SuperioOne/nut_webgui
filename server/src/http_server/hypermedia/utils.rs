const ERROR_CLASS: &str = "error";
const SUCCESS_CLASS: &str = "success";
const WARNING_CLASS: &str = "warning";
const INFO_CLASS: &str = "info";

#[macro_export]
macro_rules! htmx_redirect {
  ($c:expr, $u:expr) => {{
    let code: axum::http::StatusCode = $c;
    let uri: &str = $u;
    let headers = axum::http::HeaderMap::new();

    (code, [("HX-Redirect", uri)], headers)
  }};
}

#[derive(Debug)]
pub struct AppDetails {
  pub name: &'static str,
  pub version: &'static str,
}

pub const fn get_app_info() -> AppDetails {
  AppDetails {
    version: env!("CARGO_PKG_VERSION"),
    name: env!("CARGO_PKG_NAME"),
  }
}

pub fn get_range_class(value: &f64, from: f64, to: f64) -> &'static str {
  if from > to {
    if *value >= from {
      SUCCESS_CLASS
    } else if *value < from && *value > to {
      WARNING_CLASS
    } else {
      ERROR_CLASS
    }
  } else if *value <= from {
    SUCCESS_CLASS
  } else if *value >= from && *value < to {
    WARNING_CLASS
  } else {
    ERROR_CLASS
  }
}

#[derive(Debug)]
pub struct StatusIcon {
  pub name: &'static str,
  pub class: &'static str,
  pub desc: &'static str,
}

pub fn get_status_icons(value: &str) -> Vec<StatusIcon> {
  value
    .split_ascii_whitespace()
    .into_iter()
    .flat_map(|e| match e {
      "OL" => Some(StatusIcon {
        name: "activity",
        class: SUCCESS_CLASS,
        desc: "UPS Online",
      }),
      "BOOST" => Some(StatusIcon {
        name: "zap-off",
        class: ERROR_CLASS,
        desc: "Input voltage is too low",
      }),
      "CAL" => Some(StatusIcon {
        name: "loader",
        class: INFO_CLASS,
        desc: "UPS is calibrating",
      }),
      "CHRG" => Some(StatusIcon {
        name: "battery-charging",
        class: SUCCESS_CLASS,
        desc: "Battery is charging",
      }),
      "DISCHRG" => Some(StatusIcon {
        name: "battery",
        class: INFO_CLASS,
        desc: "Battery is discharging",
      }),
      "ALARM" => Some(StatusIcon {
        name: "alert-circle",
        class: ERROR_CLASS,
        desc: "UPS requires intervention",
      }),
      "FSD" => Some(StatusIcon {
        name: "power",
        class: ERROR_CLASS,
        desc: "FORCED SHUTDOWN has begun",
      }),
      "LB" => Some(StatusIcon {
        name: "battery",
        class: WARNING_CLASS,
        desc: "Low battery",
      }),
      "OB" => Some(StatusIcon {
        name: "battery-charging",
        class: WARNING_CLASS,
        desc: "UPS on battery",
      }),
      "OVER" => Some(StatusIcon {
        name: "alert-circle",
        class: ERROR_CLASS,
        desc: "UPS is overloaded",
      }),
      "NOCOMM" => Some(StatusIcon {
        name: "wifi-off",
        class: ERROR_CLASS,
        desc: "Daemon has no connection with UPS",
      }),
      "RB" => Some(StatusIcon {
        name: "tool",
        class: ERROR_CLASS,
        desc: "Replace battery",
      }),
      "TEST" => Some(StatusIcon {
        name: "activity",
        class: INFO_CLASS,
        desc: "Under test",
      }),
      "TRIM" => Some(StatusIcon {
        name: "zap",
        class: ERROR_CLASS,
        desc: "Input voltage is too high",
      }),
      "BYPASS" => Some(StatusIcon {
        name: "tool",
        class: INFO_CLASS,
        desc: "Battery is disconnected for maintenance",
      }),
      _ => None,
    })
    .collect()
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
    "OL" => Some(SUCCESS_CLASS),
    "BOOST" => Some(ERROR_CLASS),
    "CAL" => Some(INFO_CLASS),
    "CHRG" => Some(INFO_CLASS),
    "DISCHRG" => Some(WARNING_CLASS),
    "ALARM" => Some(ERROR_CLASS),
    "FSD" => Some(ERROR_CLASS),
    "LB" => Some(ERROR_CLASS),
    "OB" => Some(WARNING_CLASS),
    "OVER" => Some(ERROR_CLASS),
    "NOCOMM" => Some(ERROR_CLASS),
    "RB" => Some(ERROR_CLASS),
    "TEST" => Some(INFO_CLASS),
    "TRIM" => Some(ERROR_CLASS),
    "BYPASS" => Some(WARNING_CLASS),
    _ => None,
  }
}
