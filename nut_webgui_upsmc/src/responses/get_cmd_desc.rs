use crate::{
  CmdName, UpsName,
  errors::{Error, ErrorKind, ParseError},
  internal::{DeserializeResponse, ReadOnlyStr, lexer::Lexer, parser_utils::parse_line},
};

#[derive(Debug)]
pub struct CmdDesc {
  pub cmd: CmdName,
  pub ups: UpsName,
  pub desc: ReadOnlyStr,
}

impl DeserializeResponse for CmdDesc {
  type Error = Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error> {
    let (ups, cmd, desc) = parse_line!(lexer, "CMDDESC" {UPS, name = ups_name} {CMD, name = cmd_name} {QUOTED_TEXT, name = desc})?;

    if lexer.is_finished() {
      Ok(Self { cmd, ups, desc })
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
