#[derive(Debug)]
pub struct ProtVer {
  pub value: String,
}

#[derive(Debug)]
pub struct DaemonVer {
  pub value: String,
}

macro_rules! impl_ok_parser {
  ($name:ident, {$($tokens:literal)+}) => {
    #[derive(Debug)]
    pub struct $name;

    impl $crate::internal::Deserialize for $name {
      type Error = $crate::errors::Error;

      fn deserialize(lexer: &mut $crate::internal::lexer::Lexer) -> Result<Self, Self::Error> {
        use $crate::internal::parser_utils::parse_line;

        parse_line!(lexer, $($tokens)+)?;

        if lexer.is_finished() {
          Ok(Self)
        } else {
          Err(
            $crate::errors::ErrorKind::ParseError {
              inner: $crate::errors::ParseError::InvalidToken,
              position: lexer.get_positon(),
            }
            .into(),
          )
        }
      }
    }
  };
}

impl_ok_parser!(ProtOk, { "OK" });
impl_ok_parser!(ProtOkFsd, { "OK" "FSD-SET"});
impl_ok_parser!(ProtOkDetach, { "OK"  "Goodbye"});
impl_ok_parser!(ProtOkTls, { "OK" "STARTTLS"});
