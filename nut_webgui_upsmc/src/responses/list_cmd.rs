use crate::{
  CmdName, UpsName,
  errors::{Error, ErrorKind, ParseError},
  internal::{Deserialize, lexer::Lexer, parser_utils::parse_line},
};

#[derive(Debug)]
pub struct CmdList {
  pub ups_name: UpsName,
  pub cmds: Vec<CmdName>,
}

impl Deserialize for CmdList {
  type Error = Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error> {
    let mut cmds: Vec<CmdName> = Vec::new();
    let ups_name = parse_line!(lexer, "BEGIN" "LIST" "CMD" {UPS, name = ups_name})?;

    while let Some("CMD") = lexer.peek_as_str() {
      let cmd_name = parse_line!(lexer, "CMD" {TEXT, cmp_to = &ups_name} {CMD, name = cmd_name})?;

      cmds.push(cmd_name);
    }

    parse_line!(lexer, "END" "LIST" "CMD" {TEXT, cmp_to = &ups_name})?;

    if lexer.is_finished() {
      Ok(Self { ups_name, cmds })
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
