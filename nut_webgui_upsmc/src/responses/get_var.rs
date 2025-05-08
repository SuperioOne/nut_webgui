use crate::{
  UpsName, Value, VarName,
  errors::{Error, ErrorKind, ParseError},
  internal::{Deserialize, lexer::Lexer, parser_utils::parse_line},
};

#[derive(Debug)]
pub struct UpsVar {
  pub value: Value,
  pub name: VarName,
  pub ups_name: UpsName,
}

impl Deserialize for UpsVar {
  type Error = Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error> {
    let (ups_name, name, value) = parse_line!(lexer, "VAR" {UPS, name = ups_name} {VAR, name = var_name} {VALUE, name = value})?;

    if lexer.is_finished() {
      Ok(Self {
        name,
        ups_name,
        value,
      })
    } else {
      Err(
        ErrorKind::ParseError {
          inner: ParseError::InvalidToken,
          position: lexer.get_positon(),
        }
        .into(),
      )
    }
  }
}
