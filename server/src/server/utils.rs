const ERROR_CLASS: &str = "error";
const SUCCESS_CLASS: &str = "success";
const WARNING_CLASS: &str = "warning";

#[macro_export]
macro_rules! htmx_redirect {
  ($c:expr, $u:expr) => {{
    let code: axum::http::StatusCode = $c;
    let uri: &str = $u;
    let headers = axum::http::HeaderMap::new();
    (code, [("HX-Redirect", uri)], headers)
  }};
}

pub fn get_range_class(value: &u8, from: u8, to: u8) -> &'static str {
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
    "CAL" => Some(SUCCESS_CLASS),
    "CHRG" => Some(WARNING_CLASS),
    "DISCHRG" => Some(WARNING_CLASS),
    "ALARM" => Some(ERROR_CLASS),
    "FSD" => Some(ERROR_CLASS),
    "LB" => Some(ERROR_CLASS),
    "OB" => Some(WARNING_CLASS),
    "OVER" => Some(ERROR_CLASS),
    "NOCOMM" => Some(ERROR_CLASS),
    "RB" => Some(ERROR_CLASS),
    "TEST" => Some(WARNING_CLASS),
    "TRIM" => Some(ERROR_CLASS),
    "BYPASS" => Some(WARNING_CLASS),
    _ => None,
  }
}
