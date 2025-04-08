mod test_cmd_name;

use nut_webgui_upsmc::{
  _old::parser::{
    parse_cmd, parse_cmd_list, parse_ups, parse_ups_list, parse_var_list, parse_variable,
  },
  Ups,
  errors::{NutClientErrors, ParseErrorKind},
  ups_variables::{UpsError, UpsVariable},
};

#[test]
fn parse_variable_numeric() {
  let expected = UpsVariable::BatteryCharge(87.0);

  if let Ok(var) = parse_variable("VAR bx1600mi battery.charge \"87.0\"") {
    assert_eq!(expected, var);
  } else {
    assert!(false);
  }
}

#[test]
fn parse_variable_text() {
  let expected = UpsVariable::DriverName("test value".into());
  if let Ok(var) = parse_variable("VAR bx1600mi driver.name \"test value\"") {
    assert_eq!(expected, var);
  } else {
    assert!(false);
  }
}

#[test]
fn test_parse_cmd() {
  if let Ok(cmd) = parse_cmd("CMD bx1600mi beeper.disable") {
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

  if let Ok(Ups { name, desc }) = parse_ups("UPS bx1600mi \"\"") {
    assert_eq!("bx1600mi", name.as_ref());
    assert_eq!("", desc.as_ref());
  } else {
    assert!(false);
  }
}

#[test]
fn test_cmd_list() {
  let cmd_list = "BEGIN LIST CMD bx1600mi
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

  if let Ok(result) = parse_cmd_list(&cmd_list) {
    assert_eq!(12_usize, result.len());
  } else {
    assert!(false);
  }
}

#[test]
fn test_ups_list() {
  let ups_list = "BEGIN LIST UPS
UPS bx1600mi \"APC Back-UPS BX1600MI\"
END LIST UPS";

  if let Ok(result) = parse_ups_list(&ups_list) {
    assert_eq!(1_usize, result.len());
  } else {
    assert!(false);
  }
}

#[test]
fn test_var_list() {
  let var_list = "BEGIN LIST VAR bx1600mi
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

  if let Ok(result) = parse_var_list(&var_list) {
    assert_eq!(43_usize, result.len());
  } else {
    assert!(false);
  }
}

#[test]
fn parse_invalid_variable_type() {
  if let Err(NutClientErrors::ParseError {
    kind: ParseErrorKind::InvalidVarFloatFormat { inner: _ },
  }) = parse_variable("VAR bx1600mi battery.charge invalid-value")
  {
    assert!(true);
  } else {
    assert!(false, "Parser must fail when variable type is incorrect.");
  }
}

#[test]
fn parse_invalid_variable_format() {
  if let Err(NutClientErrors::ParseError {
    kind: ParseErrorKind::InvalidVarFormat,
  }) = parse_variable("NOVAR bx1600mi battery.charge")
  {
    assert!(true);
  } else {
    assert!(false, "Parser must fail when variable format is malformed.");
  }
}

#[test]
fn parse_invalid_ups_format() {
  if let Err(NutClientErrors::ParseError {
    kind: ParseErrorKind::InvalidUpsFormat,
  }) = parse_ups("VAR bx1600mi")
  {
    assert!(true);
  } else {
    assert!(false, "Parser must fail when ups format is malformed.");
  }
}

#[test]
fn parse_invalid_cmd_format() {
  if let Err(NutClientErrors::ParseError {
    kind: ParseErrorKind::InvalidCmdFormat,
  }) = parse_cmd("cmd lowercaseonly")
  {
    assert!(true);
  } else {
    assert!(false, "Parser must fail when cmd format is malformed.");
  }
}

#[test]
fn parse_invalid_var_list() {
  let input_start = "BROKEN START
VAR bx1600mi battery.charge \"100\"
VAR bx1600mi battery.charge.low \"10\"
VAR bx1600mi battery.mfr.date \"2001/01/01\"
END LIST VAR bx1600mi";

  if let Err(NutClientErrors::ParseError {
    kind: ParseErrorKind::InvalidListStart,
  }) = parse_var_list(input_start)
  {
    assert!(true);
  } else {
    assert!(false);
  }

  let input_end = "BEGIN LIST VAR bx1600mi
VAR bx1600mi battery.charge \"100\"
VAR bx1600mi battery.charge.low \"10\"
VAR bx1600mi battery.mfr.date \"2001/01/01\"";

  if let Err(NutClientErrors::ParseError {
    kind: ParseErrorKind::InvalidListEnd,
  }) = parse_var_list(input_end)
  {
    assert!(true);
  } else {
    assert!(false);
  }
}

#[test]
fn parse_invalid_ups_list() {
  let input_start = "BROKEN START
UPS bx1600mi \"APC Back-UPS BX1600MI\"
END LIST UPS";

  if let Err(NutClientErrors::ParseError {
    kind: ParseErrorKind::InvalidListStart,
  }) = parse_ups_list(input_start)
  {
    assert!(true);
  } else {
    assert!(false);
  }

  let input_end = "BEGIN LIST UPS
UPS bx1600mi \"APC Back-UPS BX1600MI\"";

  if let Err(NutClientErrors::ParseError {
    kind: ParseErrorKind::InvalidListEnd,
  }) = parse_ups_list(input_end)
  {
    assert!(true);
  } else {
    assert!(false);
  }
}

#[test]
fn parse_invalid_cmd_list() {
  let input_start = "BROKEN START
CMD bx1600mi beeper.enable
END LIST CMD bx1600mi";

  if let Err(NutClientErrors::ParseError {
    kind: ParseErrorKind::InvalidListStart,
  }) = parse_cmd_list(input_start)
  {
    assert!(true);
  } else {
    assert!(false);
  }

  let input_end = "BEGIN LIST CMD bx1600mi
CMD bx1600mi beeper.enable";

  if let Err(NutClientErrors::ParseError {
    kind: ParseErrorKind::InvalidListEnd,
  }) = parse_cmd_list(input_end)
  {
    assert!(true);
  } else {
    assert!(false);
  }
}

macro_rules! gen_protocol_error_test {
  ($test_name:ident, $parser_fn:ident) => {
    #[test]
    fn $test_name() {
      let input = "ERR DRIVER-NOT-CONNECTED";
      println!("{:?}", $parser_fn(input));

      if let Err(NutClientErrors::ProtocolError {
        kind: UpsError::DriverNotConnected,
      }) = $parser_fn(input)
      {
        assert!(true);
      } else {
        assert!(false);
      }
    }
  };
}

gen_protocol_error_test!(parse_protocol_err_cmd_list, parse_cmd_list);
gen_protocol_error_test!(parse_protocol_err_var_list, parse_var_list);
gen_protocol_error_test!(parse_protocol_err_ups_list, parse_ups_list);
