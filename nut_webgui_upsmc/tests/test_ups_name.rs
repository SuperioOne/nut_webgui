use nut_webgui_upsmc::errors::UpsNameParseError;
use nut_webgui_upsmc::{Hostname, UpsName};

macro_rules! ups_name_test {
  ($test_name:ident, $name:literal) => {
    #[test]
    fn $test_name() {
      if let Err(err) = UpsName::try_from($name) {
        assert!(false, "Command name validation failed {:?}", err)
      } else {
        assert!(true)
      }
    }
  };

  ($test_name:ident, $name:literal, $fail_reason:pat_param) => {
    #[test]
    fn $test_name() {
      match UpsName::try_from($name) {
        Err($fail_reason) => assert!(true),
        Err(err) => assert!(
          false,
          "UPS name failed but error type does not match error={:?}",
          err
        ),
        Ok(_) => assert!(
          false,
          "UPS name was expected to fail but it succeed unexpectedly"
        ),
      }
    }
  };
}

macro_rules! format_name {
(!, $($parts:tt)+) => {
    {
      struct __VALUES {
        name: Option<&'static str>,
        group: Option<&'static str>,
        hostname: Option<&'static str>,
        port: Option<u16>,
      }

      let mut result = String::new();
      let mut values = __VALUES {
        name: None,
        group: None,
        hostname: None,
        port: None,
      };

      format_name!(values, $($parts)+);

      if let Some(name) = values.name {
        result = String::from(name);
      }

      if let Some(group) = values.group {
        result = format!("{}:{}", group, result);
      }

      if let Some(hostname) = values.hostname {
        result = format!("{}@{}", result, hostname);
      }

      if let Some(port) = values.port {
        result = format!("{}:{}", result, port);
      }

      result
    }
  };

  ($target:expr, ups = $name:literal $(,$($parts:tt)+)?) => {
    $target.name = Some($name);
    $(
      format_name!($target, $($parts)+);
    )?
  };
  ($target:expr, group = $name:literal $(,$($parts:tt)+ )?) => {
    $target.group = Some($name);
    $(
      format_name!($target, $($parts)+);
    )?
  };
  ($target:expr, hostname = $name:literal $(,$($parts:tt)+)?) => {
    $target.hostname = Some($name);
    $(
      format_name!($target, $($parts)+);
    )?
  };
  ($target:expr, port = $port:literal $(,$($parts:tt)+ )?) => {
    $target.port = Some($port);
    $(
      format_name!($target, $($parts)+);
    )?
  };
}

macro_rules! verify_part {
  ($ups_name:expr, ups = $name:literal $(,$($parts:tt)+)?) => {
    assert_eq!($name, $ups_name.name.as_ref());

    $(
      verify_part!($ups_name, $($parts)+);
    )?
  };
  ($ups_name:expr, group = $group_name:literal $(,$($parts:tt)+ )?) => {
    assert_eq!(Some($group_name), $ups_name.group.as_deref());

    $(
      verify_part!($ups_name, $($parts)+);
    )?
  };
  ($ups_name:expr, hostname = $hostname:literal $(,$($parts:tt)+)?) => {
    if let Some(Hostname { name, ..}) = &$ups_name.hostname {
      assert_eq!($hostname, name.as_ref());
    }

    $(
      verify_part!($ups_name, $($parts)+);
    )?
  };
  ($ups_name:expr, port = $port:literal $(,$($parts:tt)+ )?) => {
    if let Some(hostname) = $ups_name.hostname {
      assert_eq!(Some($port), hostname.port);
    }

    $(
      verify_part!($ups_name, $($parts)+);
    )?
  };
}

macro_rules! ups_parse_test {
  ($test_name:ident, $($parts:tt)+) => {

    #[test]
    fn $test_name() {
      let input = format_name!(!, $($parts)+);
      println!("Test input {}", &input);
      let ups_name = UpsName::try_from(input.as_str()).unwrap();

      verify_part!(ups_name, $($parts)+);
    }
  };

}

ups_parse_test!(
  ups_name_all_parts,
  ups = "test",
  hostname = "localhost",
  port = 12315_u16,
  group = "group"
);

ups_parse_test!(ups_name_only, ups = "power-pulse");

ups_parse_test!(
  ups_name_with_group,
  ups = "power-pulse",
  group = "office-room-12"
);

ups_parse_test!(
  ups_name_with_hostname,
  ups = "power-pulse",
  hostname = "smdd.dev"
);

ups_parse_test!(
  ups_name_with_hostname_and_port,
  ups = "power-pulse",
  hostname = "smdd.dev",
  port = 3493_u16
);

ups_name_test!(empty_name, "", UpsNameParseError::Empty);

ups_name_test!(
  invalid_ups_name,
  "my=broken=ups",
  UpsNameParseError::InvalidUpsName
);

ups_name_test!(
  invalid_group_name,
  "=broken_group:ups_name",
  UpsNameParseError::InvalidGroupName
);

ups_name_test!(
  expected_host_name,
  "group:name@",
  UpsNameParseError::ExpectedHostName
);

ups_name_test!(min_name_len_1, "w");

#[test]
fn from_trait() {
  match UpsName::try_from("ups_name") {
    Ok(name) => assert_eq!(&name.to_string(), "ups_name"),
    Err(err) => assert!(false, "Parse failed unexpectedly error={}", err),
  }
}

#[test]
fn cmp_ups_names() {
  let lhs = UpsName::try_from("PowerPenguin").unwrap();
  let rhs = UpsName::try_from("PowerPenguin").unwrap();

  assert!(lhs == rhs)
}

#[test]
fn to_string() {
  assert_eq!(
    UpsName::try_from("protectorino").unwrap().to_string(),
    "protectorino"
  );
}
