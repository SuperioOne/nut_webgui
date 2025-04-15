use nut_webgui_upsmc::errors::UpsNameParseError;
use nut_webgui_upsmc::{Hostname, UpsName};

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
      let ups_name = UpsName::try_from(input.as_str()).unwrap();

      verify_part!(ups_name, $($parts)+);
    }
  };
}

macro_rules! ups_parse_fail_test {
  ($test_name:ident, $expected:pat_param, $($parts:tt)+) => {

    #[test]
    fn $test_name() {
      let input = format_name!(!, $($parts)+);

      match UpsName::try_from(input.as_str()) {
        Err($expected) => assert!(true),
        Err(err) => assert!(false, "Parser returned an error but error type is not equal to expected error type error={:?}", err),
        Ok(ups_name) => assert!(false, "Parser should've failed but its succeed input={} ups_name={:?}", input, ups_name),
      }
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

ups_parse_test!(ups_name_with_min_len, ups = "p");

ups_parse_fail_test!(empty_name, UpsNameParseError::Empty, ups = "");

ups_parse_fail_test!(
  invalid_ups_name,
  UpsNameParseError::InvalidUpsName,
  ups = "borked=name"
);

ups_parse_fail_test!(
  invalid_group_name,
  UpsNameParseError::InvalidGroupName,
  ups = "ups_name",
  group = "borked=group"
);

ups_parse_fail_test!(
  expected_group_name,
  UpsNameParseError::ExpectedGroupName,
  ups = "ups_name",
  group = ""
);

ups_parse_fail_test!(
  expected_group_name_2,
  UpsNameParseError::ExpectedGroupName,
  ups = "ups_name",
  hostname = "megusta.org",
  port = 9000_u16,
  group = ""
);

ups_parse_fail_test!(
  expected_host_name,
  UpsNameParseError::ExpectedHostname,
  ups = "ups_name",
  hostname = ""
);

ups_parse_fail_test!(
  expected_host_name_2,
  UpsNameParseError::ExpectedHostname,
  group = "test",
  ups = "ups_name",
  hostname = ""
);

ups_parse_fail_test!(
  expected_host_name_3,
  UpsNameParseError::ExpectedHostname,
  group = "test",
  ups = "ups_name",
  hostname = "",
  port = 42_u16
);

ups_parse_fail_test!(
  expected_ups_name,
  UpsNameParseError::ExpectedUpsName,
  ups = "",
  hostname = "megusta.org",
  port = 1616_u16
);

ups_parse_fail_test!(
  expected_ups_name_2,
  UpsNameParseError::ExpectedUpsName,
  ups = "",
  group = "group",
  hostname = "megusta.org",
  port = 1616_u16
);

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

  assert!(lhs == rhs);

  let lhs = UpsName::try_from("Group:PowerPenguin@localhost:4242").unwrap();
  let rhs = UpsName::try_from("Group:PowerPenguin@localhost:4242").unwrap();

  assert!(lhs == rhs)
}

#[test]
fn to_string() {
  assert_eq!(
    UpsName::try_from("protectorino").unwrap().to_string(),
    "protectorino"
  );
}

#[test]
fn to_string_all_parts() {
  assert_eq!(
    UpsName::try_from("group:ups_name@host:12345")
      .unwrap()
      .to_string(),
    "group:ups_name@host:12345"
  );
}

#[test]
fn to_string_with_group() {
  assert_eq!(
    UpsName::try_from("group:ups_name").unwrap().to_string(),
    "group:ups_name"
  );
}

#[test]
fn to_string_with_hostname() {
  assert_eq!(
    UpsName::try_from("ups_name@host:12345")
      .unwrap()
      .to_string(),
    "ups_name@host:12345"
  );
}
