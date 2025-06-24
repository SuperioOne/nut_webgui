use std::net::IpAddr;

use crate::{
  UpsName,
  errors::{Error, ErrorKind, ParseError},
  internal::{Deserialize, lexer::Lexer, parser_utils::parse_line},
};

#[derive(Debug)]
pub struct ClientList {
  pub ups_name: UpsName,
  pub ips: Vec<IpAddr>,
}

impl Deserialize for ClientList {
  type Error = Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error> {
    let mut ips: Vec<IpAddr> = Vec::new();
    let ups_name = parse_line!(lexer, "BEGIN" "LIST" "CLIENT" {UPS, name = ups_name})?;

    while let Some("CLIENT") = lexer.peek_as_str() {
      let ip_text = parse_line!(lexer, "CLIENT" {TEXT, cmp_to = &ups_name} {TEXT, name = ip})?;

      let ip: IpAddr = ip_text.parse().map_err(|_| ErrorKind::ParseError {
        inner: ParseError::InvalidToken,
        position: lexer.get_positon(),
      })?;

      ips.push(ip);
    }

    parse_line!(lexer, "END" "LIST" "CLIENT" {TEXT, cmp_to = &ups_name})?;

    if lexer.is_finished() {
      Ok(Self { ups_name, ips })
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
