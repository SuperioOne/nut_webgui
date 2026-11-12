use crate::internal::lexer::{Lexer, Token};

/// Basic test generator. Checks each token type and their contents against the input text.
macro_rules! tokenizer_test {
($test_name:ident, $input:literal ,[$($token:literal;)+]) => {

  #[test]
  fn $test_name(){
    let input: &str = $input;
    let mut lexer = Lexer::new(input);

    $(
      match lexer.next_token() {
        Ok(Some(token @ Token::LF { ..})) => {
          assert_eq!($token, lexer.extract_from_token(&token));
          assert_eq!($token, "\n");
        },
        Ok(Some(token @ Token::Text {..})) => {
          assert_eq!($token, lexer.extract_from_token(&token));
        },
        Ok(Some(token @ Token::QuotedText{  .. })) => {
          assert_eq!($token, lexer.extract_from_token(&token));
        },
        Ok(None) => assert!(false, "Tokenizer ended unexpectedly."),
        Err(err) => assert!(false, "Tokenizer failed unexpectedly {}", err)
      };
    )+

    let next = lexer.next_token();

    match next {
      Ok(Some(unexpected)) => assert!(false, "Tokenizer does not end properly. {:?}", unexpected),
      Err(err) => assert!(false, "Tokenizer failed at the end {}", err),
      Ok(None) => assert!(true)
    }
  }
};
  }

tokenizer_test!(starts_with_text, "hello world test.com \"test is coming\"",  [
"hello";
"world";
"test.com";
"test is coming";
]);

tokenizer_test!(starts_with_quoted_text, "\"hello world\" test.com \"test is coming\"",  [
"hello world";
"test.com";
"test is coming";
]);

tokenizer_test!(text_only, "Llanfair­pwllgwyngyll­gogery­chwyrn­drobwll­llan­tysilio­gogo­goch",  [
"Llanfair­pwllgwyngyll­gogery­chwyrn­drobwll­llan­tysilio­gogo­goch";
]);

tokenizer_test!(quoted_text_only, "\"hello world\"",  [
"hello world";
]);

tokenizer_test!(untrimmed_white_spaces, "          \"hello world\"        hello                       ",  [
"hello world";
"hello";
]);

tokenizer_test!(lines, "VAR TEST BEGIN
LINE0 \"value\"
LINE1 \"value1\"
LINE2 \"value2\"
LINE3 \"value3\"
VAR TEST END",
[
"VAR";
"TEST";
"BEGIN";
"\n";
"LINE0";
"value";
"\n";
"LINE1";
"value1";
"\n";
"LINE2";
"value2";
"\n";
"LINE3";
"value3";
"\n";
"VAR";
"TEST";
"END";
  ]);

tokenizer_test!(escaped, "\"hello \\\"world\\\"\"",  [
"hello \"world\"";
]);

tokenizer_test!(escaped_backslash, "\"\\\\some\\\\win\\\\path\"",  [
"\\some\\win\\path";
]);

tokenizer_test!(escaped_unknown, "\"\\some\\win\\path\"",  [
"somewinpath";
]);

#[test]
fn empty_text() {
  let mut lexer = Lexer::new("");

  match lexer.next_token() {
    Ok(None) => assert!(true),
    _ => assert!(false, "Input is empty, tokenizer must've return None!"),
  }
}
