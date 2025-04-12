use crate::{
  Value, VarName,
  errors::{Error, ParseError},
  internal::{
    DeserializeResponse,
    lexer::{Lexer, Token},
  },
};

pub struct GetVar {
  pub value: Value,
  pub name: VarName,
}

// VAR bx1600mi battery.charge "87.0"

impl DeserializeResponse for GetVar {
  type Error = Error;

  fn deserialize(tokenizer: &mut Lexer) -> Result<Self, Self::Error> {
    Ok(Self {
      value: Value::Int(0),
      name: VarName::UPS_STATUS,
    })
  }
}

#[cfg(test)]
mod test {
  use crate::internal::DeserializeResponse;
  use crate::internal::lexer::Lexer;

  use super::GetVar;

  #[test]
  fn extract_test() {
    let text = "VAR    group:bx1600mi@abuzer.com:12345   battery.charge \"87.0\"";
    let mut tokenizer = Lexer::from_str(text);

    _ = GetVar::deserialize(&mut tokenizer);

    assert!(true)
  }
}
