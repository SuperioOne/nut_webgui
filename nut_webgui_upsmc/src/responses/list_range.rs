use crate::{UpsName, Value, VarName};

pub struct ListRange {
  pub variable_name: VarName,
  pub min: Value,
  pub max: Value,
  pub ups: UpsName,
}
