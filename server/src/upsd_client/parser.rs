use crate::upsd_client::{errors::NutClientErrors, ups_variables::UpsVariable, Cmd, Ups, Var};

#[cfg(test)]
mod unit_tests;

#[macro_export]
macro_rules! is_error_response {
  ( $x:expr ) => {{
    let value: &str = $x;
    value.starts_with("ERR")
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
  ( $x:expr, $t:expr ) => {{
    let line: &str = $x;
    let end = format!("END LIST {}", $t);
    line.starts_with(&end)
  }};
}

#[macro_export]
macro_rules! extract_error {
  ($x:expr) => {{
    let line: &str = $x;
    $crate::upsd_client::ups_variables::UpsError::from(&line[2..])
  }};
}

macro_rules! check_list_start {
  ( $x:expr, $t:expr ) => {{
    let value: Option<&str> = $x;
    let list_type: &str = $t;
    let list_start: &str = &format!("BEGIN LIST {}", list_type);

    match value {
      Some(line) if !line.starts_with(list_start) => {
        let message = format!(
          "{0} list does not start with correct line. Unexpected line received '{1}'.",
          list_type, line
        );
        Err(NutClientErrors::ParseError(message))
      }
      Some(line) if is_error_response!(line) => {
        let error = extract_error!(&line);
        Err(NutClientErrors::ProtocolError(error))
      }
      None => {
        let message = format!(
          "{0} list does not start with correct line. Line is empty.",
          list_type
        );
        Err(NutClientErrors::ParseError(message))
      }
      _ => Ok(()),
    }
  }};
}

pub fn parse_cmd_list(buffer: &str) -> Result<Vec<Cmd>, NutClientErrors> {
  let mut line_iter = buffer.lines();
  let mut commands: Vec<Cmd> = vec![];

  check_list_start!(line_iter.next(), "CMD")?;

  while let Some(line) = line_iter.next() {
    if is_list_end!(line, "CMD") {
      return Ok(commands);
    } else {
      let cmd = parse_cmd(line)?;
      commands.push(cmd);
    }
  }

  Err(NutClientErrors::ParseError(
    "Invalid command list structure. END LIST is missing.".into(),
  ))
}

pub fn parse_ups_list(buffer: &str) -> Result<Vec<Ups>, NutClientErrors> {
  let mut line_iter = buffer.lines();
  let mut commands: Vec<Ups> = vec![];

  check_list_start!(line_iter.next(), "UPS")?;

  while let Some(line) = line_iter.next() {
    if is_list_end!(line, "UPS") {
      return Ok(commands);
    } else {
      let cmd = parse_ups(line)?;
      commands.push(cmd);
    }
  }

  Err(NutClientErrors::ParseError(
    "Invalid command list structure. END LIST is missing.".into(),
  ))
}

pub fn parse_var_list(buffer: &str) -> Result<Vec<Var>, NutClientErrors> {
  let mut line_iter = buffer.lines();
  let mut variables: Vec<Var> = vec![];

  check_list_start!(line_iter.next(), "VAR")?;

  while let Some(line) = line_iter.next() {
    if is_list_end!(line, "VAR") {
      return Ok(variables);
    } else {
      let variable = parse_variable(line)?;
      variables.push(variable);
    }
  }

  Err(NutClientErrors::ParseError(
    "Invalid command list structure. END LIST is missing.".into(),
  ))
}

pub fn parse_variable(line: &str) -> Result<Var, NutClientErrors> {
  let words = shell_words::split(line).map_err(|e| NutClientErrors::ParseError(e.to_string()))?;

  match words.as_slice() {
    [op, ups_name, var_name, value_slice] if op == "VAR" => {
      let name = Box::from(ups_name.as_str());
      let var = UpsVariable::try_from((var_name.as_str(), value_slice.as_str()))?;

      Ok(Var { var, name })
    }
    _ => Err(NutClientErrors::ParseError(
      "Unexpected variable format".into(),
    )),
  }
}

pub fn parse_cmd(line: &str) -> Result<Cmd, NutClientErrors> {
  let words = shell_words::split(line).map_err(|e| NutClientErrors::ParseError(e.to_string()))?;

  match words.as_slice() {
    [op, ups_name, cmd_name] if op == "CMD" => {
      let name = Box::from(ups_name.as_str());
      let cmd = Box::from(cmd_name.as_str());

      Ok(Cmd { name, cmd })
    }
    _ => Err(NutClientErrors::ParseError(
      "Unexpected command format".into(),
    )),
  }
}

pub fn parse_ups(line: &str) -> Result<Ups, NutClientErrors> {
  let words = shell_words::split(line).map_err(|e| NutClientErrors::ParseError(e.to_string()))?;

  match words.as_slice() {
    [op, ups_name, ups_desc] if op == "UPS" => {
      let name = Box::from(ups_name.as_str());
      let desc = Box::from(ups_desc.as_str());

      Ok(Ups { name, desc })
    }
    _ => Err(NutClientErrors::ParseError(
      "Unexpected UPS info format".into(),
    )),
  }
}
