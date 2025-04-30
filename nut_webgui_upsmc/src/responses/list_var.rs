use crate::{
  UpsName, UpsVariables,
  errors::{Error, ErrorKind, ParseError},
  internal::{Deserialize, lexer::Lexer, parser_utils::parse_line},
};
use tracing::warn;

#[derive(Debug)]
pub struct UpsVarList {
  pub variables: UpsVariables,
  pub ups: UpsName,
}

impl Deserialize for UpsVarList {
  type Error = Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error> {
    let mut variables = UpsVariables::new();
    let ups_name = parse_line!(lexer, "BEGIN" "LIST" "VAR" {TEXT, name = ups_name})?;
    let ups = UpsName::try_from(ups_name.as_ref()).map_err(|err| ErrorKind::ParseError {
      inner: ParseError::UpsName(err),
      position: lexer.get_positon(),
    })?;

    loop {
      match lexer.peek_as_str() {
        Some("VAR") => {
          let (name, value) = parse_line!(lexer, "VAR" {TEXT, cmp_to = &ups_name} {VAR, name = var_name} {VALUE, name = value})?;

          if let Some(previous) = variables.insert(name, value) {
            warn!(
              message = "variable repeated more than once in a list response",
              previous = previous.to_string(),
            )
          }
        }
        _ => break,
      }
    }

    _ = parse_line!(lexer, "END" "LIST" "VAR" {TEXT, cmp_to = &ups_name})?;

    if lexer.is_finished() {
      Ok(Self { ups, variables })
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
