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

impl DeserializeResponse for GetVar {
  type Error = Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error> {
    let (ups, name, value) = parse_line!(lexer, "VAR" {UPS, name = ups_name} {VAR, name = var_name} {VALUE, name = value})?;

    if lexer.is_finished() {
      Ok(Self { name, ups, value })
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
