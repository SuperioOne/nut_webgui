use crate::upsd_client::errors::NutClientErrors;
use crate::upsd_client::ups_variables::UpsVariable;
use crate::upsd_client::{Cmd, Ups, Var};

#[macro_export]
macro_rules! is_error_response {
  ( $x:expr ) => {{
    let value: &str = $x;
    value.starts_with("ERR")
  }};
}

#[macro_export]
macro_rules! is_ok_response {
  ( $x:expr ) => {
    "OK\n" == $x
  };
}

#[macro_export]
macro_rules! is_list_end {
  ( $x:expr ) => {{
    let line: &str = $x;
    line.starts_with("END LIST")
  }};
  ( $x:expr, $t:expr ) => {{
    let line: &str = $x;
    let end = format!("END LIST {}", $t);
    line.starts_with(&end)
  }};
}

#[macro_export]
macro_rules! extract_error {
  ($x:expr) => {{
    let line: &str = $x;
    $crate::upsd_client::ups_variables::UpsError::from(&line[2..])
  }};
}

macro_rules! check_list_start {
  ( $x:expr, $t:expr ) => {{
    let value: Option<&str> = $x;
    let list_type: &str = $t;
    let list_start: &str = &format!("BEGIN LIST {}", list_type);

    match value {
      Some(line) if !line.starts_with(list_start) => {
        let message = format!(
          "{0} list does not start with correct line. Unexpected line received '{1}'.",
          list_type, line
        );
        Err(NutClientErrors::ParseError(message))
      }
      Some(line) if is_error_response!(line) => {
        let error = extract_error!(&line);
        Err(NutClientErrors::ProtocolError(error))
      }
      None => {
        let message = format!(
          "{0} list does not start with correct line. Line is empty.",
          list_type
        );
        Err(NutClientErrors::ParseError(message))
      }
      _ => Ok(()),
    }
  }};
}

pub(crate) fn parse_cmd_list(buffer: &str) -> Result<Vec<Cmd>, NutClientErrors> {
  let mut line_iter = buffer.lines();
  let mut commands: Vec<Cmd> = vec![];

  check_list_start!(line_iter.next(), "CMD")?;

  while let Some(line) = line_iter.next() {
    if is_list_end!(line, "CMD") {
      return Ok(commands);
    } else {
      let cmd = parse_cmd(line)?;
      commands.push(cmd);
    }
  }

  Err(NutClientErrors::ParseError(
    "Invalid command list structure. END LIST is missing.".into(),
  ))
}

pub(crate) fn parse_ups_list(buffer: &str) -> Result<Vec<Ups>, NutClientErrors> {
  let mut line_iter = buffer.lines();
  let mut commands: Vec<Ups> = vec![];

  check_list_start!(line_iter.next(), "UPS")?;

  while let Some(line) = line_iter.next() {
    if is_list_end!(line, "UPS") {
      return Ok(commands);
    } else {
      let cmd = parse_ups(line)?;
      commands.push(cmd);
    }
  }

  Err(NutClientErrors::ParseError(
    "Invalid command list structure. END LIST is missing.".into(),
  ))
}

pub(crate) fn parse_var_list(buffer: &str) -> Result<Vec<Var>, NutClientErrors> {
  let mut line_iter = buffer.lines();
  let mut variables: Vec<Var> = vec![];

  check_list_start!(line_iter.next(), "VAR")?;

  while let Some(line) = line_iter.next() {
    if is_list_end!(line, "VAR") {
      return Ok(variables);
    } else {
      let variable = parse_variable(line)?;
      variables.push(variable);
    }
  }

  Err(NutClientErrors::ParseError(
    "Invalid command list structure. END LIST is missing.".into(),
  ))
}

pub(crate) fn parse_variable(line: &str) -> Result<Var, NutClientErrors> {
  let words = shell_words::split(line).map_err(|e| NutClientErrors::ParseError(e.to_string()))?;

  match words.as_slice() {
    [op, ups_name, var_name, value_slice] if op == "VAR" => {
      let name = Box::from(ups_name.as_str());
      let var = UpsVariable::try_from((var_name.as_str(), value_slice.as_str()))?;

      Ok(Var { var, name })
    }
    _ => Err(NutClientErrors::ParseError(
      "Unexpected variable format".into(),
    )),
  }
}

pub(crate) fn parse_cmd(line: &str) -> Result<Cmd, NutClientErrors> {
  let words = shell_words::split(line).map_err(|e| NutClientErrors::ParseError(e.to_string()))?;

  match words.as_slice() {
    [op, ups_name, cmd_name] if op == "CMD" => {
      let name = Box::from(ups_name.as_str());
      let cmd = Box::from(cmd_name.as_str());

      Ok(Cmd { name, cmd })
    }
    _ => Err(NutClientErrors::ParseError(
      "Unexpected command format".into(),
    )),
  }
}

pub(crate) fn parse_ups(line: &str) -> Result<Ups, NutClientErrors> {
  let words = shell_words::split(line).map_err(|e| NutClientErrors::ParseError(e.to_string()))?;

  match words.as_slice() {
    [op, ups_name, ups_desc] if op == "UPS" => {
      let name = Box::from(ups_name.as_str());
      let desc = Box::from(ups_desc.as_str());

      Ok(Ups { name, desc })
    }
    _ => Err(NutClientErrors::ParseError(
      "Unexpected UPS info format".into(),
    )),
  }
}

#[cfg(test)]
mod tests {
  use crate::upsd_client::parser::{
    parse_cmd, parse_cmd_list, parse_ups, parse_ups_list, parse_var_list, parse_variable,
  };
  use crate::upsd_client::ups_variables::UpsVariable;
  use crate::upsd_client::{Cmd, Ups, Var};

  #[test]
  fn test_parse_variable_numeric() {
    let expected = UpsVariable::BatteryCharge(87);

    if let Ok(Var { name, var }) = parse_variable("VAR bx1600mi battery.charge \"87\"") {
      assert_eq!("bx1600mi", name.as_ref());
      assert_eq!(expected, var);
    } else {
      assert!(false);
    }
  }

  #[test]
  fn test_parse_variable_text() {
    let expected = UpsVariable::DriverName("test value".into());
    if let Ok(Var { name, var }) = parse_variable("VAR bx1600mi driver.name \"test value\"") {
      assert_eq!("bx1600mi", name.as_ref());
      assert_eq!(expected, var);
    } else {
      assert!(false);
    }
  }

  #[test]
  fn test_parse_cmd() {
    if let Ok(Cmd { name, cmd }) = parse_cmd("CMD bx1600mi beeper.disable") {
      assert_eq!("bx1600mi", name.as_ref());
      assert_eq!("beeper.disable", cmd.as_ref());
    } else {
      assert!(false);
    }
  }

  #[test]
  fn test_parse_ups() {
    if let Ok(Ups { name, desc }) = parse_ups("UPS bx1600mi \"APC Back-UPS BX1600MI\"") {
      assert_eq!("bx1600mi", name.as_ref());
      assert_eq!("APC Back-UPS BX1600MI", desc.as_ref());
    } else {
      assert!(false);
    }
  }

  #[test]
  fn test_cmd_list() {
    let buffer = "BEGIN LIST CMD bx1600mi
CMD bx1600mi beeper.disable
CMD bx1600mi beeper.enable
CMD bx1600mi beeper.mute
CMD bx1600mi beeper.off
CMD bx1600mi beeper.on
CMD bx1600mi load.off
CMD bx1600mi load.off.delay
CMD bx1600mi shutdown.reboot
CMD bx1600mi shutdown.stop
CMD bx1600mi test.battery.start.deep
CMD bx1600mi test.battery.start.quick
CMD bx1600mi test.battery.stop
END LIST CMD bx1600mi";

    if let Ok(result) = parse_cmd_list(&buffer) {
      assert_eq!(12_usize, result.len());
    } else {
      assert!(false);
    }
  }

  #[test]
  fn test_ups_list() {
    let buffer = "BEGIN LIST UPS
UPS bx1600mi \"APC Back-UPS BX1600MI\"
END LIST UPS";

    if let Ok(result) = parse_ups_list(&buffer) {
      assert_eq!(1_usize, result.len());
    } else {
      assert!(false);
    }
  }

  #[test]
  fn test_var_list() {
    let buffer = "BEGIN LIST VAR bx1600mi
VAR bx1600mi battery.charge \"100\"
VAR bx1600mi battery.charge.low \"10\"
VAR bx1600mi battery.mfr.date \"2001/01/01\"
VAR bx1600mi battery.runtime \"791\"
VAR bx1600mi battery.runtime.low \"120\"
VAR bx1600mi battery.type \"PbAc\"
VAR bx1600mi battery.voltage \"27.3\"
VAR bx1600mi battery.voltage.nominal \"24.0\"
VAR bx1600mi device.mfr \"American Power Conversion\"
VAR bx1600mi device.model \"Back-UPS BX1600MI\"
VAR bx1600mi device.serial \"999999999999\"
VAR bx1600mi device.type \"ups\"
VAR bx1600mi driver.name \"usbhid-ups\"
VAR bx1600mi driver.parameter.pollfreq \"30\"
VAR bx1600mi driver.parameter.pollinterval \"1\"
VAR bx1600mi driver.parameter.port \"auto\"
VAR bx1600mi driver.parameter.productid \"0002\"
VAR bx1600mi driver.parameter.serial \"000000000000\"
VAR bx1600mi driver.parameter.synchronous \"no\"
VAR bx1600mi driver.parameter.vendorid \"051D\"
VAR bx1600mi driver.version \"2.7.4\"
VAR bx1600mi driver.version.data \"APC HID 0.96\"
VAR bx1600mi driver.version.internal \"0.41\"
VAR bx1600mi input.sensitivity \"medium\"
VAR bx1600mi input.transfer.high \"295\"
VAR bx1600mi input.transfer.low \"145\"
VAR bx1600mi input.voltage \"232.0\"
VAR bx1600mi input.voltage.nominal \"230\"
VAR bx1600mi ups.beeper.status \"enabled\"
VAR bx1600mi ups.delay.shutdown \"20\"
VAR bx1600mi ups.firmware \"378600G -302202G \"
VAR bx1600mi ups.load \"29\"
VAR bx1600mi ups.mfr \"American Power Conversion\"
VAR bx1600mi ups.mfr.date \"2023/02/25\"
VAR bx1600mi ups.model \"Back-UPS BX1600MI\"
VAR bx1600mi ups.productid \"0002\"
VAR bx1600mi ups.realpower.nominal \"900\"
VAR bx1600mi ups.serial \"000000000000\"
VAR bx1600mi ups.status \"OL\"
VAR bx1600mi ups.test.result \"Done and passed\"
VAR bx1600mi ups.timer.reboot \"0\"
VAR bx1600mi ups.timer.shutdown \"-1\"
VAR bx1600mi ups.vendorid \"051d\"
END LIST VAR bx1600mi";

    if let Ok(result) = parse_var_list(&buffer) {
      assert_eq!(43_usize, result.len());
    } else {
      assert!(false);
    }
  }
}
