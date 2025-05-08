use crate::{
  CmdName, UpsName,
  errors::{Error, ErrorKind, ParseError},
  internal::{Deserialize, ReadOnlyStr, lexer::Lexer, parser_utils::parse_line},
};

#[derive(Debug)]
pub struct CmdDesc {
  pub cmd: CmdName,
  pub ups_name: UpsName,
  pub desc: ReadOnlyStr,
}

impl Deserialize for CmdDesc {
  type Error = Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error> {
    let (ups_name, cmd, desc) = parse_line!(lexer, "CMDDESC" {UPS, name = ups_name} {CMD, name = cmd_name} {QUOTED_TEXT, name = desc})?;

    if lexer.is_finished() {
      Ok(Self {
        cmd,
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
