use crate::{
  UpsName, Value, VarName,
  error::{Error, ErrorKind, ParseError},
  internal::{Deserialize, lexer::Lexer, parser_utils::parse_line},
};

#[derive(Debug)]
pub struct EnumList {
  pub name: VarName,
  pub ups_name: UpsName,
  pub values: Vec<Value>,
}

impl Deserialize for EnumList {
  type Error = Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error> {
    let mut values: Vec<Value> = Vec::new();
    let (ups_name, var_name) =
      parse_line!(lexer, "BEGIN" "LIST" "ENUM" {UPS, name = ups_name} {TEXT, name = var_name})?;

    let name = VarName::try_from(var_name).map_err(|err| ErrorKind::ParseError {
      inner: ParseError::VarName(err),
      position: lexer.get_positon(),
    })?;

    while let Some("ENUM") = lexer.peek_as_str() {
      let value = parse_line!(lexer, "ENUM" {TEXT, cmp_to = &ups_name} {TEXT, cmp_to = name.as_str()} {VALUE, name = value})?;
      values.push(value);
    }

    parse_line!(lexer, "END" "LIST" "ENUM" {TEXT, cmp_to = &ups_name} {TEXT, cmp_to = name.as_str()})?;

    if lexer.is_finished() {
      Ok(Self {
        ups_name,
        values,
        name,
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
