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
    let ups_name_text = parse_line!(lexer, "BEGIN" "LIST" "CMD" {TEXT, name = ups_name})?;
    let ups_name =
      UpsName::try_from(ups_name_text.as_ref()).map_err(|err| ErrorKind::ParseError {
        inner: ParseError::UpsName(err),
        position: lexer.get_positon(),
      })?;

    loop {
      match lexer.peek_as_str() {
        Some("CMD") => {
          let cmd_name =
            parse_line!(lexer, "CMD" {TEXT, cmp_to = &ups_name_text} {CMD, name = cmd_name})?;

          cmds.push(cmd_name);
        }
        _ => break,
      }
    }

    _ = parse_line!(lexer, "END" "LIST" "CMD" {TEXT, cmp_to = &ups_name_text})?;

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
