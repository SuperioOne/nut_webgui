use crate::{
  UpsName, Value, VarName,
  errors::{Error, ErrorKind, ParseError},
  internal::{DeserializeResponse, lexer::Lexer, parser_utils::parse_line},
};

#[derive(Debug)]
pub struct GetVar {
  pub value: Value,
  pub name: VarName,
  pub ups: UpsName,
}

// VAR bx1600mi battery.charge "87.0"

impl DeserializeResponse for GetVar {
  type Error = Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error> {
    let (ups, name, value) = parse_line!(lexer, "VAR" {UPS, name = ups_name} {VAR, name = var_name} {QUOTED_TEXT, name = value})?;

    if lexer.is_finished() {
      Ok(Self {
        name,
        ups,
        value: Value::Int(0),
      })
    } else {
      return Err(
        ErrorKind::ParseError {
          inner: ParseError::InvalidToken,
          position: lexer.get_positon(),
        }
        .into(),
      );
    }
  }
}

#[cfg(test)]
mod test {
  use crate::internal::DeserializeResponse;
  use crate::internal::lexer::Lexer;

  use super::GetVar;

  #[test]
  fn extract_test() {
    let text = "VAR    group:bx1600mi@abuzer.com:12345   battery.charge \"87.0\"\n\n\n\n";
    let mut tokenizer = Lexer::from_str(text);

    let response = GetVar::deserialize(&mut tokenizer);

    println!("{:?}", response);

    assert!(true)
  }
}
