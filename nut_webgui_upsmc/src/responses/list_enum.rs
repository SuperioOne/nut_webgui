use crate::{
  UpsName, Value, VarName,
  errors::{Error, ErrorKind, ParseError},
  internal::{DeserializeResponse, lexer::Lexer, parser_utils::parse_line},
};

#[derive(Debug)]
pub struct EnumList {
  pub name: VarName,
  pub ups: UpsName,
  pub values: Vec<Value>,
}

impl DeserializeResponse for EnumList {
  type Error = Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error> {
    let mut values: Vec<Value> = Vec::new();
    let (ups_name, var_name) =
      parse_line!(lexer, "BEGIN" "LIST" "ENUM" {TEXT, name = ups_name} {TEXT, name = var_name})?;

    let name = VarName::new(&var_name).map_err(|err| ErrorKind::ParseError {
      inner: ParseError::VarName(err),
      position: lexer.get_positon(),
    })?;

    let ups = UpsName::try_from(ups_name.as_ref()).map_err(|err| ErrorKind::ParseError {
      inner: ParseError::UpsName(err),
      position: lexer.get_positon(),
    })?;

    loop {
      match lexer.peek_as_str() {
        Some("ENUM") => {
          let value = parse_line!(lexer, "ENUM" {TEXT, cmp_to = &ups_name} {TEXT, cmp_to = &var_name} {VALUE, name = value})?;
          values.push(value);
        }
        _ => break,
      }
    }

    _ = parse_line!(lexer, "END" "LIST" "ENUM" {TEXT, cmp_to = &ups_name} {TEXT, cmp_to = &var_name})?;

    if lexer.is_finished() {
      Ok(Self { ups, values, name })
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
