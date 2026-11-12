use crate::{InferValueFrom as _, Value};

#[test]
fn infer_float_types() {
  assert_eq!(Value::Float(-1.123), Value::infer_from("-0001.123"));
  assert_eq!(Value::Float(1.0), Value::infer_from("0001.00"));
  assert_eq!(Value::Float(1001.0), Value::infer_from("1001.00000"));
  assert_eq!(Value::Float(1.2), Value::infer_from("1.2"));
  assert_eq!(Value::Float(1.0), Value::infer_from("1.0"));
  assert_eq!(Value::Float(-1.0), Value::infer_from("-1.0"));
}

#[test]
fn infer_non_float_types() {
  assert_eq!(
    Value::String("0.001.00".into()),
    Value::infer_from("0.001.00")
  );
  assert_eq!(Value::String("1.".into()), Value::infer_from("1."));
  assert_eq!(Value::String("-1.".into()), Value::infer_from("-1."));
  assert_eq!(Value::String(".1".into()), Value::infer_from(".1"));
  assert_eq!(Value::String(".".into()), Value::infer_from("."));
}

#[test]
fn infer_str() {
  assert_eq!(Value::String("".into()), Value::infer_from(""));
  assert_eq!(
    Value::String("123456A".into()),
    Value::infer_from("123456A")
  );
  assert_eq!(Value::String("-1-".into()), Value::infer_from("-1-"));
  assert_eq!(Value::String("1.1.1".into()), Value::infer_from("1.1.1"));
}

#[test]
fn infer_int() {
  assert_eq!(Value::Int(-1234), Value::infer_from("-1234"));
  assert_eq!(Value::Int(1234), Value::infer_from("1234"));
  assert_eq!(Value::Int(0009123), Value::infer_from("0009123"));
  assert_eq!(Value::Int(0), Value::infer_from("-0"));
  assert_eq!(Value::Int(-1), Value::infer_from("-1"));
  assert_eq!(Value::Int(0), Value::infer_from("0"));
}
