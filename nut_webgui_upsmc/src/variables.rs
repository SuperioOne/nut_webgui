use crate::value::Value;
use crate::var_name::VarName;

/// UPS variable key/value struct.
#[derive(Debug)]
pub struct UpsVar(pub VarName, pub Value);

/// List of UPS variables.
pub struct UpsVariables {
  inner: Vec<UpsVar>,
}
