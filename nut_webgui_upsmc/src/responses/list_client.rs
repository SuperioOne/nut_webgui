use std::net::IpAddr;

use crate::{
  UpsName,
  errors::{Error, ErrorKind, ParseError},
  internal::{Deserialize, lexer::Lexer, parser_utils::parse_line},
};

#[derive(Debug)]
pub struct ClientList {
  pub ups: UpsName,
  pub ips: Vec<IpAddr>,
}

impl Deserialize for ClientList {
  type Error = Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error> {
    let mut ips: Vec<IpAddr> = Vec::new();
    let ups_name = parse_line!(lexer, "BEGIN" "LIST" "CLIENT" {TEXT, name = ups_name})?;
    let ups = UpsName::try_from(ups_name.as_ref()).map_err(|err| ErrorKind::ParseError {
      inner: ParseError::UpsName(err),
      position: lexer.get_positon(),
    })?;

    loop {
      match lexer.peek_as_str() {
        Some("CLIENT") => {
          let ip_text = parse_line!(lexer, "CLIENT" {TEXT, cmp_to = &ups_name} {TEXT, name = ip})?;

          let ip: IpAddr = ip_text.parse().map_err(|_| ErrorKind::ParseError {
            inner: ParseError::InvalidToken,
            position: lexer.get_positon(),
          })?;

          ips.push(ip);
        }
        _ => break,
      }
    }

    _ = parse_line!(lexer, "END" "LIST" "CLIENT" {TEXT, cmp_to = &ups_name})?;

    if lexer.is_finished() {
      Ok(Self { ups, ips })
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
