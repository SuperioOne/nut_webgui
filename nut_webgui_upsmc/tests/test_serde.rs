#![cfg(feature = "serde")]

use nut_webgui_upsmc::{
  CmdName, Hostname, UpsName, Value, VarName, ups_status::UpsStatus, variables::UpsVariables,
};
use std::num::NonZeroU16;

#[test]
fn cmd_name() {
  let input: Vec<CmdName> = vec![
    CmdName::new_unchecked("test0"),
    CmdName::new_unchecked("test1"),
    CmdName::new_unchecked("test2"),
    CmdName::new_unchecked("test3"),
    CmdName::new_unchecked("test4"),
  ];

  let json_str = serde_json::to_string_pretty(&input).unwrap();
  let deserialized: Vec<CmdName> = serde_json::from_str(json_str.as_str()).unwrap();

  assert_eq!(input.len(), deserialized.len());

  for (r, l) in input.into_iter().zip(deserialized) {
    assert_eq!(r, l);
  }
}

#[test]
fn ups_name() {
  let input: Vec<UpsName> = vec![
    UpsName::new_unchecked("test0").set_hostname(
      Hostname::new("localhost").set_port(unsafe { NonZeroU16::new_unchecked(9090) }),
    ),
    UpsName::new_unchecked("test1"),
    UpsName::new_unchecked("test2").set_hostname(Hostname::new("localhost")),
    UpsName::new_unchecked("backup_sample").set_group_unchecked("sector_e"),
    UpsName::new_unchecked("lambda_core"),
  ];

  let json_str = serde_json::to_string_pretty(&input).unwrap();
  println!("{json_str}");

  let deserialized: Vec<UpsName> = serde_json::from_str(json_str.as_str()).unwrap();

  assert_eq!(input.len(), deserialized.len());

  for (r, l) in input.into_iter().zip(deserialized) {
    assert_eq!(r, l);
  }
}

#[test]
fn variables() {
  let mut input = UpsVariables::new();

  input.insert(VarName::UPS_STATUS, Value::from("enabled"));
  input.insert(VarName::BATTERY_VOLTAGE, Value::from(12.0));
  input.insert(VarName::INPUT_LOAD, Value::from(52));
  input.insert(
    VarName::new_unchecked("custom.variable.name"),
    Value::from("0X123456"),
  );

  let json_str = serde_json::to_string_pretty(&input).unwrap();
  let deserialized: UpsVariables = serde_json::from_str(json_str.as_str()).unwrap();

  assert_eq!(input.len(), deserialized.len());

  for (name, value) in input.iter() {
    match deserialized.get(name) {
      Some(v) => assert_eq!(v, value),
      None => assert!(false, "missing variable {name}"),
    }
  }
}

#[test]
fn ups_status() {
  let input = vec![
    UpsStatus::ONLINE,
    UpsStatus::ON_BATTERY | UpsStatus::DISCHARGE,
  ];

  let json_str = serde_json::to_string_pretty(&input).unwrap();
  let deserialized: Vec<UpsStatus> = serde_json::from_str(json_str.as_str()).unwrap();

  assert_eq!(input.len(), deserialized.len());

  for (r, l) in input.iter().zip(deserialized.iter()) {
    assert_eq!(r, l);
  }
}
