use crate::{Value, VarName};

pub struct ListRange {
  pub variable_name: VarName,
  pub min: Value,
  pub max: Value,
}
