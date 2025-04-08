use crate::errors::NutClientErrors;
use crate::{Value, VarName};

use super::DeserializeResponse;

pub struct GetVar {
  pub value: Value,
  pub name: VarName,
}

// VAR bx1600mi battery.charge "87.0"

impl DeserializeResponse for GetVar {
  type Error = NutClientErrors;

  fn deserialize(bytes: &str) -> Result<Self, Self::Error> {
    if bytes.is_empty() {
      return Err(NutClientErrors::EmptyResponse);
    }

    Ok(Self {
      value: Value::Int(0),
      name: VarName::UPS_STATUS,
    })
  }
}
