use nut_webgui_upsmc::VarName;
use nut_webgui_upsmc::errors::VarNameParseError;

macro_rules! var_name_test {
  ($test_name:ident, $name:literal) => {
    #[test]
    fn $test_name() {
      if let Err(err) = VarName::new($name) {
        assert!(false, "Variable name validation failed {:?}", err)
      } else {
        assert!(true)
      }
    }
  };

  ($test_name:ident, $name:literal, $fail_reason:pat_param) => {
    #[test]
    fn $test_name() {
      match VarName::new($name) {
        Err($fail_reason) => assert!(true),
        Err(err) => assert!(
          false,
          "Variable name failed but error type does not match error={:?}",
          err
        ),
        Ok(_) => assert!(
          false,
          "Variable name was expected to fail but it succeed unexpectedly"
        ),
      }
    }
  };
}

macro_rules! var_cmp_test {
  ($test_name:ident, $left:expr, $right:expr) => {
    #[test]
    fn $test_name() {
      assert!($left.eq(&$right))
    }
  };
}

var_name_test!(empty_name, "", VarNameParseError::Empty);

var_name_test!(
  invalid_chars,
  "ups.var;with.semicolon",
  VarNameParseError::InvalidName
);

var_name_test!(
  invalid_whitespace,
  "  ups.not.trimmed",
  VarNameParseError::InvalidName
);

var_name_test!(
  invalid_dot_at_start,
  ".ups.not.trimmed",
  VarNameParseError::InvalidName
);

var_name_test!(
  invalid_number_at_start,
  "1ups.status",
  VarNameParseError::InvalidName
);

var_name_test!(
  invalid_char_at_start,
  "=ups.status",
  VarNameParseError::InvalidName
);

var_name_test!(valid_name, "ups.load");

var_name_test!(valid_name_with_numeric, "ups.sensors.1.temperature");

var_name_test!(min_name_len_1, "w");

var_cmp_test!(
  cmp_standard_name,
  VarName::UPS_STATUS,
  VarName::new("ups.status").unwrap()
);

var_cmp_test!(
  cmp_standard_name_reflexivity,
  VarName::new("ups.status").unwrap(),
  VarName::UPS_STATUS
);

var_cmp_test!(
  cmp_custom_names,
  VarName::new("custom.name").unwrap(),
  VarName::new("custom.name").unwrap()
);

#[test]
fn from_trait() {
  match VarName::try_from("ups.status") {
    Ok(name) => assert_eq!(name, VarName::UPS_STATUS),
    Err(err) => assert!(false, "Parse failed unexpectedly error={}", err),
  }

  match VarName::try_from("1ups.status") {
    Err(VarNameParseError::InvalidName) => assert!(true),
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
fn to_string() {
  assert_eq!(
    VarName::new("custom.name.1").unwrap().as_str(),
    "custom.name.1"
  );

  assert_eq!(
    VarName::new("custom.name.1").unwrap().to_string(),
    String::from("custom.name.1")
  );

  assert_eq!(
    VarName::AMBIENT_TEMPERATURE_LOW_WARNING.as_str(),
    "ambient.temperature.low.warning"
  );

  assert_eq!(
    VarName::AMBIENT_TEMPERATURE_LOW_WARNING.to_string(),
    String::from("ambient.temperature.low.warning")
  );
}
