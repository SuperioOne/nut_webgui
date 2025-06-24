use crate::{
  UpsName,
  errors::{Error, ErrorKind, ParseError},
  internal::{Deserialize, lexer::Lexer, parser_utils::parse_line},
};

#[derive(Debug)]
pub struct UpsDevice {
  pub ups_name: UpsName,
  pub desc: Box<str>,
}

#[derive(Debug)]
pub struct UpsList {
  pub devices: Vec<UpsDevice>,
}

impl Deserialize for UpsList {
  type Error = Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error> {
    let mut devices: Vec<UpsDevice> = Vec::new();
    parse_line!(lexer, "BEGIN" "LIST" "UPS")?;

    while let Some("UPS") = lexer.peek_as_str() {
      let (ups_name, desc) =
        parse_line!(lexer, "UPS" {UPS, name = ups_name} {QUOTED_TEXT, name = desc})?;

      devices.push(UpsDevice { ups_name, desc });
    }

    parse_line!(lexer, "END" "LIST" "UPS")?;

    if lexer.is_finished() {
      Ok(Self { devices })
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
