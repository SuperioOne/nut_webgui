use crate::{
  UpsName,
  errors::{Error, ErrorKind, ParseError},
  internal::{DeserializeResponse, ReadOnlyStr, lexer::Lexer, parser_utils::parse_line},
};

#[derive(Debug)]
pub struct UpsDevice {
  pub ups: UpsName,
  pub desc: ReadOnlyStr,
}

#[derive(Debug)]
pub struct UpsList {
  pub devices: Vec<UpsDevice>,
}

impl DeserializeResponse for UpsList {
  type Error = Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error> {
    let mut devices: Vec<UpsDevice> = Vec::new();
    _ = parse_line!(lexer, "BEGIN" "LIST" "UPS")?;

    loop {
      match lexer.peek_as_str() {
        Some("UPS") => {
          let (ups, desc) =
            parse_line!(lexer, "UPS" {UPS, name = ups_name} {QUOTED_TEXT, name = desc})?;

          devices.push(UpsDevice { ups, desc });
        }
        _ => break,
      }
    }

    _ = parse_line!(lexer, "END" "LIST" "UPS")?;

    if lexer.is_finished() {
      Ok(Self { devices })
    } else {
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
