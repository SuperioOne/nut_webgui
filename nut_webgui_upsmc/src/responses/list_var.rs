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
    let ups_name = parse_line!(lexer, "BEGIN" "LIST" "VAR" {UPS, name = ups_name})?;

    while let Some("VAR") = lexer.peek_as_str() {
      let (name, value) = parse_line!(lexer, "VAR" {TEXT, cmp_to = &ups_name} {VAR, name = var_name} {VALUE, name = value})?;

      if let Some(previous) = variables.insert(name, value) {
        warn!(
          message = "variable repeated more than once in a list response",
          previous = previous.to_string(),
        )
      }
    }

    parse_line!(lexer, "END" "LIST" "VAR" {TEXT, cmp_to = &ups_name})?;

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
