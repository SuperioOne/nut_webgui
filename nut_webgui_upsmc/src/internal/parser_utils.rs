use crate::{
  CmdName, UpsName, Value, VarName,
  errors::{Error, ErrorKind, ParseError},
  internal::lexer::{Lexer, Token},
};
use std::borrow::Cow;
use tracing::trace;

pub fn extract_text<'a>(lexer: &'a mut Lexer) -> Result<Cow<'a, str>, Error> {
  match lexer.next_token()? {
    Some(token @ Token::Text { .. }) => Ok(lexer.extract_from_token(&token)),
    Some(token) => {
      trace!(
        message = "trying to extract text but received a different kind of token",
        kind = token.kind().as_str()
      );

      Err(
        ErrorKind::ParseError {
          inner: ParseError::InvalidToken,
          position: lexer.get_positon(),
        }
        .into(),
      )
    }
    None => {
      trace!(message = "trying to extract text, but received nothing from lexer");

      Err(
        ErrorKind::ParseError {
          inner: ParseError::ExpectedTextToken,
          position: lexer.get_positon(),
        }
        .into(),
      )
    }
  }
}

pub fn extract_quoted_text<'a>(lexer: &'a mut Lexer) -> Result<Cow<'a, str>, Error> {
  match lexer.next_token()? {
    Some(token @ Token::QuotedText { .. }) => Ok(lexer.extract_from_token(&token)),
    Some(token) => {
      trace!(
        message = "trying to extract quoted-text, but received a different kind of token",
        kind = token.kind().as_str()
      );

      Err(
        ErrorKind::ParseError {
          inner: ParseError::InvalidToken,
          position: lexer.get_positon(),
        }
        .into(),
      )
    }
    None => {
      trace!(message = "trying to extract quoted-text, but received nothing from lexer");

      Err(
        ErrorKind::ParseError {
          inner: ParseError::ExpectedDoubleQuotedTextToken,
          position: lexer.get_positon(),
        }
        .into(),
      )
    }
  }
}

pub fn extract_value<'a>(lexer: &'a mut Lexer) -> Result<Value, Error> {
  let text = extract_quoted_text(lexer)?;
  let value = Value::infer_from_str(&text);

  Ok(value)
}

pub fn extract_ups_name<'a>(lexer: &'a mut Lexer) -> Result<UpsName, Error> {
  match lexer.next_token()? {
    Some(token @ Token::Text { .. }) => {
      let name = lexer.extract_from_token(&token);

      let ups_name = UpsName::try_from(name.as_ref()).map_err(|err| ErrorKind::ParseError {
        inner: ParseError::UpsName(err),
        position: lexer.get_positon(),
      })?;

      Ok(ups_name)
    }
    Some(token) => {
      trace!(
        message =
          "trying to extract text token for ups name, but received a different kind of token",
        kind = token.kind().as_str()
      );

      Err(
        ErrorKind::ParseError {
          inner: ParseError::InvalidToken,
          position: lexer.get_positon(),
        }
        .into(),
      )
    }
    None => {
      trace!(message = "trying to extract ups name, but received nothing from lexer");

      Err(
        ErrorKind::ParseError {
          inner: ParseError::ExpectedDoubleQuotedTextToken,
          position: lexer.get_positon(),
        }
        .into(),
      )
    }
  }
}

pub fn extract_cmd_name<'a>(lexer: &'a mut Lexer) -> Result<CmdName, Error> {
  match lexer.next_token()? {
    Some(token @ Token::Text { .. }) => {
      let name = lexer.extract_from_token(&token);

      let cmd_name = CmdName::try_from(name.as_ref()).map_err(|err| ErrorKind::ParseError {
        inner: ParseError::CmdName(err),
        position: lexer.get_positon(),
      })?;

      Ok(cmd_name)
    }
    Some(token) => {
      trace!(
        message = "trying to extract text token for cmd, but received a different kind of token",
        kind = token.kind().as_str()
      );

      Err(
        ErrorKind::ParseError {
          inner: ParseError::InvalidToken,
          position: lexer.get_positon(),
        }
        .into(),
      )
    }
    None => {
      trace!(message = "trying to extract cmd, but received nothing from lexer");

      Err(
        ErrorKind::ParseError {
          inner: ParseError::ExpectedTextToken,
          position: lexer.get_positon(),
        }
        .into(),
      )
    }
  }
}

pub fn extract_var_name<'a>(lexer: &'a mut Lexer) -> Result<VarName, Error> {
  match lexer.next_token()? {
    Some(token @ Token::Text { .. }) => {
      let name = lexer.extract_from_token(&token);

      let var_name = VarName::try_from(name.as_ref()).map_err(|err| ErrorKind::ParseError {
        inner: ParseError::VarName(err),
        position: lexer.get_positon(),
      })?;

      Ok(var_name)
    }
    Some(token) => {
      trace!(
        message =
          "trying to extract text token for variable name, but received a different kind of token",
        kind = token.kind().as_str()
      );

      Err(
        ErrorKind::ParseError {
          inner: ParseError::InvalidToken,
          position: lexer.get_positon(),
        }
        .into(),
      )
    }
    None => {
      trace!(message = "trying to extract variable name, but received nothing from lexer");

      Err(
        ErrorKind::ParseError {
          inner: ParseError::ExpectedTextToken,
          position: lexer.get_positon(),
        }
        .into(),
      )
    }
  }
}

pub fn cmp_literal<'a>(lexer: &'a mut Lexer, cmp: &str) -> Result<(), Error> {
  match lexer.next_token()? {
    Some(token @ Token::Text { .. }) => {
      let name = lexer.extract_from_token(&token);

      if name == cmp {
        Ok(())
      } else {
        trace!(
          message = "literal tokens does not match",
          received = name.as_ref(),
          expected = cmp
        );

        Err(
          ErrorKind::ParseError {
            inner: ParseError::InvalidToken,
            position: lexer.get_positon(),
          }
          .into(),
        )
      }
    }
    Some(token) => {
      trace!(
        message =
          "trying to extract text token for variable name, but received a different kind of token",
        kind = token.kind().as_str()
      );

      Err(
        ErrorKind::ParseError {
          inner: ParseError::InvalidToken,
          position: lexer.get_positon(),
        }
        .into(),
      )
    }
    None => {
      trace!(message = "trying to extract variable name, but received nothing from lexer");

      Err(
        ErrorKind::ParseError {
          inner: ParseError::ExpectedTextToken,
          position: lexer.get_positon(),
        }
        .into(),
      )
    }
  }
}

pub fn end_parser<'a>(lexer: &'a mut Lexer) -> Result<(), Error> {
  match lexer.next_token()? {
    Some(Token::LF) => {
      // Skip all remaining whitespaces,
      lexer.skip_whitespaces();
      Ok(())
    }
    Some(token) => {
      trace!(
        message = "lexer still contains unprocessed tokens",
        next = token.kind().as_str()
      );

      Err(
        ErrorKind::ParseError {
          inner: ParseError::InvalidToken,
          position: lexer.get_positon(),
        }
        .into(),
      )
    }
    None => Ok(()),
  }
}

/// Iterates over lexer and tries to parse a line based on given syntax definition.
///
/// ## Example usage:
/// ```ignore
/// let mut lexer = Lexer::new("VAR bx1600mi ups.beeper.status \"enabled\"");
///
/// let (ups, var, value) = parse_line!(&mut lexer, "VAR" {UPS, name = ups_name} {VAR, name = var_name} {QUOTED_TEXT, name = value})?;
/// _ = parse_line!(&mut lexer, "END" "VAR" "LIST" {TEXT})?;
/// ```
///
/// ## Return
/// Return type is [`Result<(..), Error>`].
///
/// ## Limitations and Gotchas:
/// - Each capture must have an unique name due to how the macro is implemented.
/// - When a line is parsed, parser will skip/eat all remaning ASCII whitespaces (including LFs) from the lexer automatically.
/// - Unlike the trimming at the end, [`parse_line!()`] does not skip [`Token::LF`] tokens at the start.
/// - The elements of the returned tuple follow the order specified in the syntax definition (FIFO).
/// - Specialized names can only be captured using their specific syntax constructs. When the entire name isn't required for parsing, use the generic `{TEXT}` syntax instead.
///
/// ## Syntax:
///
/// `"LITERAL"` : Compares text token against a [`&'static str`].
/// `{CMD, name = extract_name}` : Captures text token and tries to parse it as [`CmdName`].
/// `{VAR, name = extract_name}` : Captures text token and tries to parse it as [`VarName`].
/// `{UPS, name = extract_name}` : Captures text token and tries to parse it as [`UpsName`].
/// `{TEXT, name = extract_name}`  : Captures text token.
/// `{TEXT, cmp_to = { "compareable expression" }}` : Compares text token against an expression.
/// `{TEXT}` : Only checks if the token type is [`Token::Text`].
/// `{QUOTED_TEXT, name = extract_name}` : Captures quoted text token.
/// `{QUOTED_TEXT}` : Only checks if token type is [`Token::QuotedText`]
/// `{VALUE, name = extracted_name}` : Captures [`Token::QuotedText`] and converts it to [`Value`] with type inference.
macro_rules! parse_line {
  ($lexer:expr, $($rules:tt)+) => {
    'line: {
      parse_line!(@internal $lexer, 'line, [], $($rules)+)
    }
  };

  (@internal $lexer:expr, $label:lifetime, [$($type:ty;$extracted:ident)*], $token:literal $($rest:tt)*) => {
    {
      if let Err(err) = $crate::internal::parser_utils::cmp_literal($lexer, $token) {
        break $label Err(err);
      }

      parse_line!(@internal $lexer, $label, [$($type;$extracted)*], $($rest)*)
    }
  };

  (@internal $lexer:expr, $label:lifetime, [$($type:ty;$extracted:ident)*], {UPS, name = $name:ident} $($rest:tt)*) => {
    {
      let $name = {
        match $crate::internal::parser_utils::extract_ups_name($lexer) {
          Ok(ups_name) => ups_name,
          Err(err) => break $label Err(err)
        }
      };

      parse_line!(@internal $lexer, $label, [$($type;$extracted)* $crate::UpsName;$name], $($rest)*)
    }
  };

  (@internal $lexer:expr,$label:lifetime, [$($type:ty;$extracted:ident)*], {TEXT, cmp_to = $cmp:expr} $($rest:tt)*) => {
    {
      if let Err(err) = $crate::internal::parser_utils::cmp_literal($lexer, $cmp) {
        break $label Err(err);
      }

      parse_line!(@internal $lexer, $label, [$($type;$extracted)*], $($rest)*)
    }
  };

  (@internal $lexer:expr,$label:lifetime, [$($type:ty;$extracted:ident)*], {QUOTED_TEXT} $($rest:tt)*) => {
    {
      match $crate::internal::parser_utils::extract_quoted_text($lexer) {
        Ok(_) => parse_line!(@internal $lexer, $label, [$($type;$extracted)*], $($rest)*),
        Err(err) => break $label Err(err)
      }
    }
  };

  (@internal $lexer:expr,$label:lifetime, [$($type:ty;$extracted:ident)*], {TEXT} $($rest:tt)*) => {
    {
      if let Err(err) = $crate::internal::parser_utils::extract_text($lexer) {
        break $label Err(err);
      }

      parse_line!(@internal $lexer, $label, [$($type;$extracted)*], $($rest)*)
    }
  };

  (@internal $lexer:expr,$label:lifetime, [$($type:ty;$extracted:ident)*], {QUOTED_TEXT} $($rest:tt)*) => {
    {
      match $crate::internal::parser_utils::extract_quoted_text($lexer) {
        Ok(_) => parse_line!(@internal $lexer, $label, [$($type;$extracted)*], $($rest)*),
        Err(err) => break $label Err(err)
      }
    }
  };

  (@internal $lexer:expr,$label:lifetime, [$($type:ty;$extracted:ident)*], {TEXT, name = $name:ident} $($rest:tt)*) => {
    {
      let $name = {
        match $crate::internal::parser_utils::extract_text($lexer) {
          Ok(std::borrow::Cow::Borrowed(text)) => $crate::internal::ReadOnlyStr::from(text),
          Ok(std::borrow::Cow::Owned(text)) => text.into_boxed_str(),
          Err(err) => break $label Err(err)
        }
      };

      parse_line!(@internal $lexer, $label, [$($type;$extracted)* $crate::internal::ReadOnlyStr;$name], $($rest)*)
    }
  };

  (@internal $lexer:expr, $label:lifetime, [$($type:ty;$extracted:ident)*], {QUOTED_TEXT, name = $name:ident} $($rest:tt)*) => {
    {
      let $name = {
        match $crate::internal::parser_utils::extract_quoted_text($lexer) {
          Ok(std::borrow::Cow::Borrowed(text)) => $crate::internal::ReadOnlyStr::from(text),
          Ok(std::borrow::Cow::Owned(text)) => text.into_boxed_str(),
          Err(err) => break $label Err(err)
        }
      };

      parse_line!(@internal $lexer, $label, [$($type;$extracted)* $crate::internal::ReadOnlyStr;$name], $($rest)*)
    }
  };

  (@internal $lexer:expr, $label:lifetime, [$($type:ty;$extracted:ident)*], {CMD, name = $name:ident} $($rest:tt)*) => {
    {
      let $name = {
        match $crate::internal::parser_utils::extract_cmd_name($lexer) {
          Ok(cmd) => cmd,
          Err(err) => break $label Err(err)
        }
      };

      parse_line!(@internal $lexer, $label, [$($type;$extracted)* $crate::CmdName;$name], $($rest)*)
    }
  };

  (@internal $lexer:expr, $label:lifetime, [$($type:ty;$extracted:ident)*], {VAR, name = $name:ident} $($rest:tt)*) => {
    {
      let $name = {
        match $crate::internal::parser_utils::extract_var_name($lexer) {
          Ok(var) => var,
          Err(err) => break $label Err(err)
        }
      };

      parse_line!(@internal $lexer, $label, [$($type;$extracted)* $crate::VarName;$name], $($rest)*)
    }
  };

  (@internal $lexer:expr, $label:lifetime, [$($type:ty;$extracted:ident)*], {VALUE, name = $name:ident} $($rest:tt)*) => {
    {
      let $name = {
        match $crate::internal::parser_utils::extract_value($lexer) {
          Ok(var) => var,
          Err(err) => break $label Err(err)
        }
      };

      parse_line!(@internal $lexer, $label, [$($type;$extracted)* $crate::Value;$name], $($rest)*)
    }
  };

  (@internal $lexer:expr,$label:lifetime, [],) => {
    {
      if let Err(err) = $crate::internal::parser_utils::end_parser($lexer) {
        break $label Err(err);
      }

      Result::<(), $crate::errors::Error>::Ok(())
    }
  };

  (@internal $lexer:expr,$label:lifetime, [$type:ty;$extracted:ident],) => {
    {
      if let Err(err) = $crate::internal::parser_utils::end_parser($lexer) {
        break $label Err(err);
      }

      Result::<$type, $crate::errors::Error>::Ok($extracted)
    }
  };

  (@internal $lexer:expr,$label:lifetime, [$type_first:ty;$extracted_first:ident $($type:ty;$extracted:ident)+],) => {
    {
      if let Err(err) = $crate::internal::parser_utils::end_parser($lexer) {
        break $label Err(err);
      }

      Result::<($type_first, $($type),+), $crate::errors::Error>::Ok(($extracted_first, $($extracted),+))
    }
  };
}

pub(crate) use parse_line;
