use std::str::FromStr;

use crate::{
  UpsName, VarName, VarType,
  errors::{Error, ErrorKind, ParseError},
  internal::{
    Deserialize,
    lexer::{Lexer, Token},
    parser_utils::{cmp_literal, extract_ups_name, extract_var_name},
  },
};

#[derive(Debug)]
pub struct UpsVarType {
  pub var_types: Vec<VarType>,
  pub name: VarName,
  pub ups_name: UpsName,
}

impl Deserialize for UpsVarType {
  type Error = Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error> {
    let mut var_types = Vec::new();

    _ = cmp_literal(lexer, "TYPE")?;
    let ups_name = extract_ups_name(lexer)?;
    let name = extract_var_name(lexer)?;

    while let Some(token) = lexer.next_token()? {
      match token {
        Token::Text { .. } => {
          let type_info = VarType::from_str(&lexer.extract_from_token(&token)).map_err(|err| {
            ErrorKind::ParseError {
              inner: ParseError::VarType(err),
              position: lexer.get_positon(),
            }
          })?;

          var_types.push(type_info);
        }
        Token::LF => break,
        _ => {
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

    if lexer.is_finished() {
      Ok(Self {
        name,
        ups_name,
        var_types,
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
