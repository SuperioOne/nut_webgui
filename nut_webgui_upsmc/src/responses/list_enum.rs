use crate::{UpsName, Value, VarName};

pub struct ListEnum {
  pub variable_name: VarName,
  pub values: Vec<Value>,
  pub ups: UpsName,
}
