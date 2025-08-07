use core::str::FromStr;
use nut_webgui_upsmc::UpsName;
use nut_webgui_upsmc::error::UpsNameParseError;

macro_rules! ups_name_test {
  ($test_name:ident, $name:literal) => {
    #[test]
    fn $test_name() {
      if let Err(err) = UpsName::new($name) {
        assert!(false, "Ups name validation failed {:?}", err)
      } else {
        assert!(true)
      }
    }
  };

  ($test_name:ident, $name:literal, $fail_reason:pat_param) => {
    #[test]
    fn $test_name() {
      match UpsName::new($name) {
        Err($fail_reason) => assert!(true),
        Err(err) => assert!(
          false,
          "Ups name failed but error type does not match error={:?}",
          err
        ),
        Ok(_) => assert!(
          false,
          "Ups name was expected to fail but it succeed unexpectedly"
        ),
      }
    }
  };
}

ups_name_test!(empty_name, "", UpsNameParseError::Empty);

ups_name_test!(
  invalid_whitespace,
  "ups with white spaces",
  UpsNameParseError::InvalidName
);

ups_name_test!(
  invalid_whitespace_at_start,
  "  sector_e",
  UpsNameParseError::InvalidName
);

ups_name_test!(valid_name, "nuclear_reactor_1");
ups_name_test!(
  non_rfc_spec_valid_name,
  "$_\"[yes,this_name_does_work_on_nut_server!]\"_$"
);

ups_name_test!(min_name_len_1, "w");

#[test]
fn from_trait() {
  match UpsName::from_str("regularups") {
    Ok(name) => assert_eq!(name, "regularups"),
    Err(err) => assert!(false, "Parse failed unexpectedly error={}", err),
  }

  match UpsName::from_str("regular ups") {
    Err(UpsNameParseError::InvalidName) => assert!(true),
    Err(err) => assert!(
      false,
      "Expected error but error type is not correct error={}",
      err
    ),
    Ok(name) => assert!(false, "Expected error but got ups name name={:?}", name),
  }
}

#[test]
fn cmp_ups_names() {
  let lhs = UpsName::new("test_name").unwrap();
  let rhs = UpsName::new("test_name").unwrap();

  assert!(lhs == rhs)
}

#[test]
fn to_string() {
  assert_eq!(UpsName::new("custom.ups").unwrap(), "custom.ups");

  assert_eq!(
    UpsName::new("custom.ups").unwrap().to_string(),
    String::from("custom.ups")
  );
}
