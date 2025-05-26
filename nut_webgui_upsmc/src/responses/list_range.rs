use crate::{
  UpsName, Value, VarName,
  errors::{Error, ErrorKind, ParseError},
  internal::{Deserialize, lexer::Lexer, parser_utils::parse_line},
};

#[derive(Debug)]
pub struct RangeList {
  pub ups_name: UpsName,
  pub ranges: Vec<(Value, Value)>,
  pub name: VarName,
}

impl Deserialize for RangeList {
  type Error = Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error> {
    let mut ranges: Vec<(Value, Value)> = Vec::new();
    let (ups_name_text, var_name) =
      parse_line!(lexer, "BEGIN" "LIST" "RANGE" {TEXT, name = ups_name} {TEXT, name = var_name})?;

    let name = VarName::try_from(var_name).map_err(|err| ErrorKind::ParseError {
      inner: ParseError::VarName(err),
      position: lexer.get_positon(),
    })?;

    let ups_name =
      UpsName::try_from(ups_name_text.as_ref()).map_err(|err| ErrorKind::ParseError {
        inner: ParseError::UpsName(err),
        position: lexer.get_positon(),
      })?;

    loop {
      match lexer.peek_as_str() {
        Some("RANGE") => {
          let range = parse_line!(lexer, "RANGE" {TEXT, cmp_to = &ups_name_text} {TEXT, cmp_to = name.as_str()} {VALUE, name = min} {VALUE, name = max})?;
          ranges.push(range);
        }
        _ => break,
      }
    }

    _ = parse_line!(lexer, "END" "LIST" "RANGE" {TEXT, cmp_to = &ups_name_text} {TEXT, cmp_to = name.as_str()})?;

    if lexer.is_finished() {
      Ok(Self {
        ups_name,
        ranges,
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
