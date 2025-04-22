use crate::{
  UpsName,
  errors::{Error, ErrorKind, ParseError},
  internal::{DeserializeResponse, ReadOnlyStr, lexer::Lexer, parser_utils::parse_line},
};

#[derive(Debug)]
pub struct UpsDesc {
  pub desc: ReadOnlyStr,
  pub ups: UpsName,
}

impl DeserializeResponse for UpsDesc {
  type Error = Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error> {
    let (ups, desc) =
      parse_line!(lexer, "UPSDESC" {UPS, name = ups_name} {QUOTED_TEXT, name = desc})?;

    if lexer.is_finished() {
      Ok(Self { ups, desc })
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
