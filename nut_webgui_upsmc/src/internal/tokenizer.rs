#[derive(Debug)]
pub enum Token {
  At(usize),
  Backslash(usize),
  Colon(usize),
  DoubleQuote(usize),
  LF(usize),
  Text(usize, usize),
  Whitespace(usize),
}

pub struct TokenIterator<'a> {
  text_bytes: &'a [u8],
  slice_head: usize,
  read_head: usize,
}

impl<'a> Iterator for TokenIterator<'a> {
  type Item = Token;

  fn next(&mut self) -> Option<Self::Item> {
    macro_rules! char_token_case {
      ($variant:ident) => {{
        if self.read_head != 0 && self.slice_head < self.read_head {
          let token = Some(Token::Text(self.slice_head, self.read_head - 1));

          self.slice_head = self.read_head + 1;

          return token;
        } else {
          let token = Some(Token::$variant(self.read_head));
          self.read_head += 1;
          self.slice_head = self.read_head;

          return token;
        }
      }};
    }

    while self.read_head < self.text_bytes.len() {
      let char_byte = self.text_bytes.get(self.read_head);

      match char_byte.as_deref() {
        Some(b'"') => char_token_case!(DoubleQuote),
        Some(b':') => char_token_case!(Colon),
        Some(b'@') => char_token_case!(At),
        Some(b'\\') => char_token_case!(Backslash),
        Some(b'\n') => char_token_case!(LF),
        Some(v) if v.is_ascii_whitespace() => char_token_case!(Whitespace),
        _ => {
          self.read_head += 1;
        }
      }
    }

    if self.slice_head < self.text_bytes.len() {
      let token = Some(Token::Text(self.slice_head, self.text_bytes.len() - 1));
      self.slice_head = self.text_bytes.len();

      token
    } else {
      None
    }
  }
}

pub trait StringTokenizer {
  fn iter_tokenizer(&self) -> TokenIterator<'_>;
}

impl StringTokenizer for &str {
  fn iter_tokenizer(&self) -> TokenIterator<'_> {
    TokenIterator {
      text_bytes: self.as_bytes(),
      slice_head: 0,
      read_head: 0,
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::internal::tokenizer::{StringTokenizer, Token};

  fn has_any_special_token(input: &str) -> bool {
    for chr_byte in input.as_bytes() {
      match *chr_byte {
        b'"' => return true,
        b':' => return true,
        b'@' => return true,
        b'\\' => return true,
        b'\n' => return true,
        v if v.is_ascii_whitespace() => return true,
        _ => continue,
      }
    }

    false
  }

  /// Basic test generator. Checks each token type and their contents against the input text.
  macro_rules! tokenizer_test {
    ($test_name:ident, [$($text:literal;)+]) => {

      #[test]
      fn $test_name(){
        let input: &str = concat!($($text,)+);
        let mut tokenizer = input.iter_tokenizer();

        $(
          match tokenizer.next() {
           Some(Token::At(idx)) => {
             assert_eq!($text, &input[idx..=idx]);
             assert_eq!($text, "@");
           },
           Some(Token::Backslash(idx)) => {
             assert_eq!($text, &input[idx..=idx]);
             assert_eq!($text, "\\");
           },
           Some(Token::Colon(idx)) => {
             assert_eq!($text, &input[idx..=idx]);
             assert_eq!($text, ":");
           },
           Some(Token::DoubleQuote(idx)) => {
             assert_eq!($text, &input[idx..=idx]);
             assert_eq!($text, "\"");
           },
           Some(Token::LF(idx)) => {
             assert_eq!($text, &input[idx..=idx]);
             assert_eq!($text, "\n");
           },
           Some(Token::Whitespace(idx)) => {
             let slice = &input[idx..=idx];

             assert_eq!($text, slice);

             if let Some(char_byte) = slice.as_bytes().get(0) {
               assert!(char_byte.is_ascii_whitespace());
             } else {
                assert!(false, "Whitespace token does not contain any whitespace character.");
             }
           },
           Some(Token::Text(start, end)) => {
             assert_eq!($text, &input[start..=end]);
             assert_eq!(has_any_special_token($text), false, "Text token not supposed to have any special characters {}.", $text);
           },
            _ => assert!(false, "Tokenizer ended unexpectedly.")
          };
        )+

        if let None = tokenizer.next() {
          assert!(true)
        } else {
          assert!(false, "Tokenizer does not end properly.");
        }
      }
    };
  }

  tokenizer_test!(starts_with_text, [
    "hello";
    ":";
    "world";
    "@";
    "test.com";
    ":";
    "2345";
  ]);

  tokenizer_test!(starts_with_special_token, [
    "@";
    "world";
    "@";
    "test.com";
    ":";
    "2345";
  ]);

  tokenizer_test!(starts_with_special_tokens, [
    " ";
    ":";
    "world";
    "@";
    "test.com";
    ":";
    "2345";
  ]);

  tokenizer_test!(ends_with_special_tokens, [
    "world";
    "@";
    "test.com";
    ":";
    "\n";
  ]);

  tokenizer_test!(ends_with_special_token, [
    "world";
    "@";
    "test.com";
    ":";
  ]);

  tokenizer_test!(special_tokens_only, [
    " ";
    "\n";
    ":";
    "@";
    "\\";
    "\"";
  ]);

  tokenizer_test!(text_only, [
    "hello.world";
  ]);

  tokenizer_test!(lines, [
    "Line1";
    "\n";
    "Line2";
    "\n";
  ]);

  tokenizer_test!(words, [
    "word1";
    " ";
    "word2";
    " ";
    "word3.";
  ]);

  #[test]
  fn empty_text() {
    for _ in "".iter_tokenizer() {
      assert!(false, "Input is empty, tokenizer must've return None!");
    }

    assert!(true);
  }
}
