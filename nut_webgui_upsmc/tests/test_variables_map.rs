use nut_webgui_upsmc::{Value, VarName, variables::UpsVariables};

#[test]
fn initialize() {
  let vars = UpsVariables::from([
    (VarName::UPS_STATUS, Value::String("OL".into())),
    (VarName::UPS_BEEPER_STATUS, Value::String("enabled".into())),
    (VarName::INPUT_LOAD, Value::Float(12.0)),
  ]);

  assert_eq!(vars.len(), 3);
}

#[test]
fn remove() {
  let mut vars = UpsVariables::from([
    (VarName::UPS_STATUS, Value::String("OL".into())),
    (VarName::UPS_BEEPER_STATUS, Value::String("enabled".into())),
    (VarName::INPUT_LOAD, Value::Float(12.0)),
  ]);

  match vars.remove(VarName::UPS_STATUS) {
    Some((name, value)) => {
      assert_eq!(name, VarName::UPS_STATUS);
      assert_eq!(value, Value::String("OL".into()));
    }
    None => assert!(false, "Unable to find and remove value from hashmap"),
  };

  match vars.remove(VarName::UPS_STATUS) {
    None => assert!(true),
    _ => assert!(false, "Variable is not removed"),
  };

  assert_eq!(vars.len(), 2);
}

#[test]
fn insert_new() {
  let mut vars = UpsVariables::from([
    (VarName::UPS_STATUS, Value::String("OL".into())),
    (VarName::UPS_BEEPER_STATUS, Value::String("enabled".into())),
    (VarName::INPUT_LOAD, Value::Float(12.0)),
  ]);

  match vars.insert(VarName::UPS_LOAD, Value::Float(100.0)) {
    Some(value) => {
      assert!(
        false,
        "Insert should've not return any value for insert new test. value={}",
        value
      );
    }
    None => assert!(true),
  };

  match vars.get(VarName::UPS_LOAD) {
    Some(value) => {
      assert_eq!(&Value::Float(100.0), value);
    }
    None => assert!(false, "Unable to read new value."),
  };

  assert_eq!(vars.len(), 4);
}

#[test]
fn update() {
  let mut vars = UpsVariables::from([
    (VarName::UPS_STATUS, Value::String("OL".into())),
    (VarName::UPS_BEEPER_STATUS, Value::String("enabled".into())),
    (VarName::INPUT_LOAD, Value::Float(12.0)),
  ]);

  match vars.insert(VarName::UPS_STATUS, Value::String("OB".into())) {
    Some(value) => {
      assert_eq!(Value::String("OL".into()), value);
    }
    None => assert!(false, "Insert should've return value for update test."),
  };

  match vars.get(VarName::UPS_STATUS) {
    Some(value) => {
      assert_eq!(&Value::String("OB".into()), value);
    }
    None => assert!(false, "Unable to read updated value."),
  };

  assert_eq!(vars.len(), 3);
}
