use nut_webgui_upsmc::CmdName;
use nut_webgui_upsmc::errors::CmdParseError;

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

cmd_name_test!(empty_name, "", CmdParseError::Empty);

cmd_name_test!(
  invalid_chars,
  "ups.cmd;with.semicolon",
  CmdParseError::InvalidName
);

cmd_name_test!(
  invalid_whitespace,
  "  battery.off",
  CmdParseError::InvalidName
);

cmd_name_test!(
  invalid_dot_at_start,
  ".ups.shutdown",
  CmdParseError::InvalidName
);

cmd_name_test!(valid_name, "beeper.off");

cmd_name_test!(min_name_len_1, "w");

#[test]
fn from_trait() {
  match CmdName::try_from("beeper.off") {
    Ok(name) => assert_eq!(name.as_str(), "beeper.off"),
    Err(err) => assert!(false, "Parse failed unexpectedly error={}", err),
  }

  match CmdName::try_from("0beeper.off") {
    Err(CmdParseError::InvalidName) => assert!(true),
    Err(err) => assert!(
      false,
      "Expected error but error type is not correct error={}",
      err
    ),
    Ok(name) => assert!(
      false,
      "Expected error but got variable name name={:?}",
      name
    ),
  }
}

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
