use crate::{
  UpsName,
  errors::{Error, ErrorKind, ParseError},
  internal::{Deserialize, lexer::Lexer, parser_utils::parse_line},
  variables::UpsVariables,
};
use tracing::warn;

#[derive(Debug)]
pub struct UpsVarList {
  pub variables: UpsVariables,
  pub ups_name: UpsName,
}

impl Deserialize for UpsVarList {
  type Error = Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error> {
    let mut variables = UpsVariables::new();
    let ups_name_text = parse_line!(lexer, "BEGIN" "LIST" "VAR" {TEXT, name = ups_name})?;
    let ups_name =
      UpsName::try_from(ups_name_text.as_ref()).map_err(|err| ErrorKind::ParseError {
        inner: ParseError::UpsName(err),
        position: lexer.get_positon(),
      })?;

    loop {
      match lexer.peek_as_str() {
        Some("VAR") => {
          let (name, value) = parse_line!(lexer, "VAR" {TEXT, cmp_to = &ups_name_text} {VAR, name = var_name} {VALUE, name = value})?;

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

    _ = parse_line!(lexer, "END" "LIST" "VAR" {TEXT, cmp_to = &ups_name_text})?;

    if lexer.is_finished() {
      Ok(Self {
        ups_name,
        variables,
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
