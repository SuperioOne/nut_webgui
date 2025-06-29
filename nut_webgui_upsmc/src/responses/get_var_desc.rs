use crate::{
  UpsName, VarName,
  errors::{Error, ErrorKind, ParseError},
  internal::{Deserialize, lexer::Lexer, parser_utils::parse_line},
};

#[derive(Debug)]
pub struct UpsVarDesc {
  pub name: VarName,
  pub desc: Box<str>,
  pub ups_name: UpsName,
}

impl Deserialize for UpsVarDesc {
  type Error = Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error> {
    let (ups_name, name, desc) = parse_line!(lexer, "DESC" {UPS, name = ups_name} {VAR, name = var_name} {QUOTED_TEXT, name = desc})?;

    if lexer.is_finished() {
      Ok(Self {
        name,
        ups_name,
        desc,
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
