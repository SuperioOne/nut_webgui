use nut_webgui_upsmc::CmdName;
use nut_webgui_upsmc::errors::ParseErrors;

macro_rules! cmd_name_test {
  ($test_name:ident, $name:literal) => {
    #[test]
    fn $test_name() {
      if let Err(err) = CmdName::new($name) {
        assert!(false, "Command name validation failed {:?}", err)
      } else {
        assert!(true)
      }
    }
  };

  ($test_name:ident, $name:literal, $fail_reason:pat_param) => {
    #[test]
    fn $test_name() {
      match CmdName::new($name) {
        Err($fail_reason) => assert!(true),
        Err(err) => assert!(
          false,
          "Command name failed but error type does not match error={:?}",
          err
        ),
        Ok(_) => assert!(
          false,
          "Command name was expected to fail but it succeed unexpectedly"
        ),
      }
    }
  };
}

cmd_name_test!(empty_name, "", ParseErrors::Empty);

cmd_name_test!(
  invalid_chars,
  "ups.cmd;with.semicolon",
  ParseErrors::InvalidChar { position: 7 }
);

cmd_name_test!(
  invalid_whitespace,
  "  battery.off",
  ParseErrors::InvalidChar { position: 0 }
);

cmd_name_test!(
  invalid_dot_at_start,
  ".ups.shutdown",
  ParseErrors::InvalidChar { position: 0 }
);

cmd_name_test!(
  invalid_uppercase,
  "battery.tesT",
  ParseErrors::InvalidChar { position: 11 }
);

cmd_name_test!(valid_name, "beeper.off");

cmd_name_test!(
  invalid_name_with_numeric,
  "ups.sensors.1.on",
  ParseErrors::InvalidChar { position: 12 }
);

cmd_name_test!(min_name_len_1, "w");

#[test]
fn cmp_cmd_names() {
  let lhs = CmdName::new("custom.name").unwrap();
  let rhs = CmdName::new("custom.name").unwrap();

  assert!(lhs == rhs)
}

#[test]
fn to_string() {
  assert_eq!(CmdName::new("custom.cmd").unwrap().as_str(), "custom.cmd");

  assert_eq!(
    CmdName::new("custom.cmd").unwrap().to_string(),
    String::from("custom.cmd")
  );
}
