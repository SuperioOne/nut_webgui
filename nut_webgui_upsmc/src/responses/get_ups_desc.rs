use crate::{
  UpsName,
  errors::{Error, ErrorKind, ParseError},
  internal::{Deserialize, lexer::Lexer, parser_utils::parse_line},
};

#[derive(Debug)]
pub struct UpsDesc {
  pub desc: Box<str>,
  pub ups_name: UpsName,
}

impl Deserialize for UpsDesc {
  type Error = Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error> {
    let (ups_name, desc) =
      parse_line!(lexer, "UPSDESC" {UPS, name = ups_name} {QUOTED_TEXT, name = desc})?;

    if lexer.is_finished() {
      Ok(Self { ups_name, desc })
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
