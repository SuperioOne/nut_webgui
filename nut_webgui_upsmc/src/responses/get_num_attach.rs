use crate::{
  UpsName,
  errors::{Error, ErrorKind, ParseError},
  internal::{Deserialize, lexer::Lexer, parser_utils::parse_line},
};

#[derive(Debug)]
pub struct AttachedDaemons {
  pub ups: UpsName,
  pub attached: usize,
}

impl Deserialize for AttachedDaemons {
  type Error = Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error> {
    let (ups, value) =
      parse_line!(lexer, "NUMATTACH" {UPS, name = ups_name} {TEXT, name = attached})?;

    let attached = value.parse::<usize>().map_err(|_| ErrorKind::ParseError {
      inner: ParseError::InvalidToken,
      position: lexer.get_positon(),
    })?;

    if lexer.is_finished() {
      Ok(Self { ups, attached })
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
