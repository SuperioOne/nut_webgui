use crate::{
  UpsName,
  errors::{Error, ErrorKind, ParseError},
  internal::{Deserialize, lexer::Lexer, parser_utils::parse_line},
  variables::UpsVariables,
};
use tracing::warn;

#[derive(Debug)]
pub struct RwList {
  pub variables: UpsVariables,
  pub ups_name: UpsName,
}

impl Deserialize for RwList {
  type Error = Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error> {
    let mut variables = UpsVariables::new();
    let ups_name = parse_line!(lexer, "BEGIN" "LIST" "RW" {UPS, name = ups_name})?;

    loop {
      match lexer.peek_as_str() {
        Some("RW") => {
          let (name, value) = parse_line!(lexer, "RW" {TEXT, cmp_to = &ups_name} {VAR, name = var_name} {VALUE, name = value})?;

          if let Some(previous) = variables.insert(name, value) {
            warn!(
              message = "rw variable repeated more than once in a list response",
              previous = previous.to_string(),
            )
          }
        }
        _ => break,
      }
    }

    _ = parse_line!(lexer, "END" "LIST" "RW" {TEXT, cmp_to = &ups_name})?;

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
