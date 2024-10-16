use super::errors::ParseErrorKind;
use crate::upsd_client::{errors::NutClientErrors, ups_variables::UpsVariable, Ups};
use tracing::error;

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

pub fn parse_variable(line: &str) -> Result<UpsVariable, NutClientErrors> {
  let words = shell_words::split(line).map_err(|err| {
    error!(message = "Unable to split variable line.", reason = %err);
    NutClientErrors::from(ParseErrorKind::InvalidVarFormat)
  })?;

  match words.as_slice() {
    [op, _ups_name, var_name, value_slice] if op == "VAR" => {
      let var = UpsVariable::try_from((var_name.as_str(), value_slice.as_str()))?;

      Ok(var)
    }
    _ => Err(NutClientErrors::from(ParseErrorKind::InvalidVarFormat)),
  }
}

pub fn parse_cmd(line: &str) -> Result<Box<str>, NutClientErrors> {
  let words = shell_words::split(line).map_err(|err| {
    error!(message = "Unable to split cmd line.", reason = %err);
    NutClientErrors::from(ParseErrorKind::InvalidCmdFormat)
  })?;

  match words.as_slice() {
    [op, _ups_name, cmd_name] if op == "CMD" => {
      let cmd = Box::from(cmd_name.as_str());

      Ok(cmd)
    }
    _ => Err(NutClientErrors::from(ParseErrorKind::InvalidCmdFormat)),
  }
}

pub fn parse_ups(line: &str) -> Result<Ups, NutClientErrors> {
  let words = shell_words::split(line).map_err(|err| {
    error!(message = "Unable to split ups line.", reason = %err);
    NutClientErrors::from(ParseErrorKind::InvalidUpsFormat)
  })?;

  match words.as_slice() {
    [op, ups_name, ups_desc] if op == "UPS" => {
      let name = Box::from(ups_name.as_str());
      let desc = Box::from(ups_desc.as_str());

      Ok(Ups { name, desc })
    }
    _ => Err(NutClientErrors::from(ParseErrorKind::InvalidUpsFormat)),
  }
}
