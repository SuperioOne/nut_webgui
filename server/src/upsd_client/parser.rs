use super::errors::ParseErrorKind;
use crate::upsd_client::{errors::NutClientErrors, ups_variables::UpsVariable, Ups};
use std::borrow::Cow;

#[cfg(test)]
mod unit_tests;

#[macro_export]
macro_rules! is_error_response {
  ( $x:expr ) => {{
    let value: &str = $x;
    value.starts_with("ERR ")
  }};
}

#[macro_export]
macro_rules! is_ok_response {
  ( $x:expr ) => {
    "OK\n" == $x
  };
}

#[macro_export]
macro_rules! is_list_end {
  ( $x:expr ) => {{
    let line: &str = $x;
    line.starts_with("END LIST")
  }};
  ( $x:expr, $t:literal ) => {{
    let line: &str = $x;
    line.starts_with(concat!("END LIST ", $t))
  }};
}

#[macro_export]
macro_rules! extract_error {
  ($x:expr) => {{
    let line: &str = $x;
    $crate::upsd_client::ups_variables::UpsError::from(&line[4..])
  }};
}

macro_rules! check_list_start {
  ( $x:expr, $t:literal) => {{
    match $x {
      Some(line) if is_error_response!(line) => {
        let error = extract_error!(&line);
        Err(NutClientErrors::from(error))
      }
      Some(line) if !line.starts_with(concat!("BEGIN LIST ", $t)) => {
        Err(NutClientErrors::from(ParseErrorKind::InvalidListStart))
      }
      None => Err(NutClientErrors::EmptyResponse),
      _ => Ok(()),
    }
  }};
}

const ESCAPE_BYTE: u8 = b'\\';
const ESCAPE_CHAR: char = '\\';
const QUOTE_BYTE: u8 = b'"';

struct AsciiWords<'a> {
  inner: Vec<Cow<'a, str>>,
}

// NOTE: Insert `Must Not` meme to limit the urge for creating SIMD version.

/// Splits US-ASCII text into words based on notation defined in RFC 9271 with minimal allocations.
impl<'a> AsciiWords<'a> {
  /// Internal only
  /// If necessary, allocates new String without escape characters.
  fn into_escaped(word: &'a str) -> Cow<'a, str> {
    if let Some(first_idx) = word.find(ESCAPE_CHAR) {
      let mut escaped_word = String::from(&word[..first_idx]);
      let mut slice_start: usize = 0;
      let remaining = &word[first_idx + 1..];

      for (idx, character) in remaining.as_bytes().iter().enumerate() {
        if *character == ESCAPE_BYTE {
          if slice_start < idx {
            escaped_word.push_str(&remaining[slice_start..idx]);
            slice_start = idx + 1;
          }
        }
      }

      if slice_start < remaining.len() {
        escaped_word.push_str(&remaining[slice_start..]);
      }

      Cow::Owned(escaped_word)
    } else {
      Cow::Borrowed(word)
    }
  }

  pub fn split(input: &'a str) -> Self {
    let mut words: Vec<Cow<'a, str>> = Vec::new();
    let mut slice_start: Option<usize> = None;
    let mut escape: bool = false;
    let mut slice_char: Option<u8> = None;

    for (idx, char_byte) in input.as_bytes().iter().enumerate() {
      if escape {
        escape = false;
      } else if *char_byte == QUOTE_BYTE && slice_char != Some(QUOTE_BYTE) {
        slice_char = Some(QUOTE_BYTE);
        slice_start = Some(idx + 1);
      } else if slice_char.is_some_and(|sc| sc == *char_byte)
        || (slice_char.is_none() && char_byte.is_ascii_whitespace())
      {
        if let Some(start) = slice_start {
          words.push(Self::into_escaped(&input[start..idx]));
        }

        slice_start = None;
        slice_char = None;
      } else if *char_byte == ESCAPE_BYTE {
        escape = true;

        if slice_start.is_none() {
          slice_start = Some(idx);
        }
      } else if slice_start.is_none() {
        slice_start = Some(idx);
      }
    }

    if let Some(start) = slice_start {
      words.push(Self::into_escaped(&input[start..]));
    }

    Self { inner: words }
  }

  /// Returns splitted words as slice.
  #[inline]
  pub fn as_slice(&self) -> &[Cow<'a, str>] {
    &self.inner
  }
}

#[inline]
pub fn parse_cmd_list(buffer: &str) -> Result<Vec<Box<str>>, NutClientErrors> {
  let mut line_iter = buffer.lines();
  let mut commands: Vec<Box<str>> = Vec::new();

  check_list_start!(line_iter.next(), "CMD")?;

  for line in line_iter {
    if is_list_end!(line, "CMD") {
      return Ok(commands);
    } else {
      let cmd = parse_cmd(line)?;
      commands.push(cmd);
    }
  }

  Err(NutClientErrors::from(ParseErrorKind::InvalidListEnd))
}

#[inline]
pub fn parse_ups_list(buffer: &str) -> Result<Vec<Ups>, NutClientErrors> {
  let mut line_iter = buffer.lines();
  let mut commands: Vec<Ups> = Vec::new();

  check_list_start!(line_iter.next(), "UPS")?;

  for line in line_iter {
    if is_list_end!(line, "UPS") {
      return Ok(commands);
    } else {
      let cmd = parse_ups(line)?;
      commands.push(cmd);
    }
  }

  Err(NutClientErrors::from(ParseErrorKind::InvalidListEnd))
}

#[inline]
pub fn parse_var_list(buffer: &str) -> Result<Vec<UpsVariable>, NutClientErrors> {
  let mut line_iter = buffer.lines();
  let mut variables: Vec<UpsVariable> = Vec::new();

  check_list_start!(line_iter.next(), "VAR")?;

  for line in line_iter {
    if is_list_end!(line, "VAR") {
      return Ok(variables);
    } else {
      let variable = parse_variable(line)?;
      variables.push(variable);
    }
  }

  Err(NutClientErrors::from(ParseErrorKind::InvalidListEnd))
}

#[inline]
pub fn parse_variable(line: &str) -> Result<UpsVariable, NutClientErrors> {
  let words = AsciiWords::split(line);

  match words.as_slice() {
    [op, _ups_name, var_name, value_slice] if op == "VAR" => {
      let var = UpsVariable::try_from((var_name.as_ref(), value_slice.as_ref()))?;

      Ok(var)
    }
    _ => Err(NutClientErrors::from(ParseErrorKind::InvalidVarFormat)),
  }
}

#[inline]
pub fn parse_cmd(line: &str) -> Result<Box<str>, NutClientErrors> {
  let words = AsciiWords::split(line);

  match words.as_slice() {
    [op, _ups_name, cmd_name] if op == "CMD" => {
      let cmd = Box::from(cmd_name.as_ref());

      Ok(cmd)
    }
    _ => Err(NutClientErrors::from(ParseErrorKind::InvalidCmdFormat)),
  }
}

#[inline]
pub fn parse_ups(line: &str) -> Result<Ups, NutClientErrors> {
  let words = AsciiWords::split(line);

  match words.as_slice() {
    [op, ups_name, ups_desc] if op == "UPS" => {
      let name = Box::from(ups_name.as_ref());
      let desc = Box::from(ups_desc.as_ref());

      Ok(Ups { name, desc })
    }
    _ => Err(NutClientErrors::from(ParseErrorKind::InvalidUpsFormat)),
  }
}
