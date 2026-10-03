#[derive(Clone, Copy)]
pub struct ReaderParseError;

#[derive(Clone, Copy, Default, Debug)]
pub struct ReaderInfo {
  pub pid: isize,
  pub thread: usize,
  pub txnid: Option<usize>,
}

// Brain-dead parser for reader list output.
pub fn parse_reader_list(text: &str) -> Result<Vec<ReaderInfo>, ReaderParseError> {
  let mut line_iter = text.lines();

  match line_iter.next() {
    Some("(no active readers)") | Some("(no reader locks)") | None => return Ok(Vec::new()),
    Some(line) => {
      if !line.trim_start().starts_with("pid") {
        return Err(ReaderParseError);
      }
    }
  };

  let mut result = Vec::new();
  for line in line_iter {
    let mut column_iter = line.split_ascii_whitespace();
    let mut info = ReaderInfo::default();

    match column_iter.next() {
      Some(pid) => info.pid = pid.parse().map_err(|_| ReaderParseError)?,
      None => return Err(ReaderParseError),
    }

    match column_iter.next() {
      Some(thread) => {
        info.thread = usize::from_str_radix(thread, 16).map_err(|_| ReaderParseError)?
      }
      None => return Err(ReaderParseError),
    }

    match column_iter.next() {
      Some("-") => info.txnid = None,
      Some(txnid) => info.txnid = Some(txnid.parse().map_err(|_| ReaderParseError)?),
      None => return Err(ReaderParseError),
    }

    if let Some(_) = column_iter.next() {
      return Err(ReaderParseError);
    }

    result.push(info);
  }

  Ok(result)
}
