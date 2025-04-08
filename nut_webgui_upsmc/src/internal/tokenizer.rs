#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
  At(usize),
  Backslash(usize),
  Colon(usize),
  DoubleQuote(usize),
  LF(usize),
  Text(usize, usize),
  Whitespace(usize),
}

#[derive(Debug, Clone, Copy)]
struct TokenizerState {
  slice_head: usize,
  read_head: usize,
}

pub struct Tokenizer<'a> {
  buffer: &'a [u8],
  state: TokenizerState,
}

impl<'a> Tokenizer<'a> {
  pub const fn new(buffer: &'a [u8]) -> Self {
    Self {
      buffer,
      state: TokenizerState {
        read_head: 0,
        slice_head: 0,
      },
    }
  }

  #[inline]
  pub const fn from_str(buffer: &'a str) -> Self {
    Self::new(buffer.as_bytes())
  }

  pub fn skip_while<F>(&mut self, predicate: F)
  where
    F: Fn((Token, Option<Token>)) -> bool,
  {
    let mut tmp_tokenizer = Self {
      state: self.state,
      buffer: self.buffer,
    };

    let mut tmp_state = tmp_tokenizer.state;

    loop {
      match tmp_tokenizer.next_token() {
        Some(token) => {
          let next = tmp_tokenizer.peek();

          if predicate((token, next)) {
            tmp_state = tmp_tokenizer.state;
          } else {
            self.state = tmp_state;
            break;
          }
        }
        None => {
          break;
        }
      }
    }
  }

  pub fn take_while<F>(&mut self, predicate: F) -> Vec<Token>
  where
    F: Fn((Token, Option<Token>)) -> bool,
  {
    let mut extracted: Vec<Token> = Vec::new();
    let mut tmp_tokenizer = Self {
      state: self.state,
      buffer: self.buffer,
    };

    let mut tmp_state = tmp_tokenizer.state;

    loop {
      match tmp_tokenizer.next_token() {
        Some(token) => {
          let next = tmp_tokenizer.peek();

          if predicate((token, next)) {
            tmp_state = tmp_tokenizer.state;
            extracted.push(token);
          } else {
            self.state = tmp_state;
            break;
          }
        }
        None => {
          break;
        }
      }
    }

    extracted
  }

  pub fn skip_whitespace(&mut self) {
    let mut tmp_tokenizer = Self {
      state: self.state,
      buffer: self.buffer,
    };

    let mut state = self.state;

    while let Some(Token::Whitespace(_)) = tmp_tokenizer.next_token() {
      state = tmp_tokenizer.state;
      continue;
    }

    self.state = state;
  }

  pub fn peek(&self) -> Option<Token> {
    let mut tmp_tokenizer = Self {
      state: self.state,
      buffer: self.buffer,
    };

    tmp_tokenizer.next_token()
  }

  pub fn next_token(&mut self) -> Option<Token> {
    macro_rules! char_token_case {
      ($variant:ident) => {{
        if self.state.read_head != 0 && self.state.slice_head < self.state.read_head {
          let token = Some(Token::Text(self.state.slice_head, self.state.read_head - 1));

          self.state.slice_head = self.state.read_head + 1;

          return token;
        } else {
          let token = Some(Token::$variant(self.state.read_head));
          self.state.read_head += 1;
          self.state.slice_head = self.state.read_head;

          return token;
        }
      }};
    }

    while self.state.read_head < self.buffer.len() {
      let char_byte = self.buffer.get(self.state.read_head);

      match char_byte.as_deref() {
        Some(b'"') => char_token_case!(DoubleQuote),
        Some(b':') => char_token_case!(Colon),
        Some(b'@') => char_token_case!(At),
        Some(b'\\') => char_token_case!(Backslash),
        Some(b'\n') => char_token_case!(LF),
        Some(v) if v.is_ascii_whitespace() => char_token_case!(Whitespace),
        _ => {
          self.state.read_head += 1;
        }
      }
    }

    if self.state.slice_head < self.buffer.len() {
      let token = Some(Token::Text(self.state.slice_head, self.buffer.len() - 1));
      self.state.slice_head = self.buffer.len();

      token
    } else {
      None
    }
  }
}

pub struct TokenizerIter<'a> {
  inner: Tokenizer<'a>,
}

impl<'a> Iterator for TokenizerIter<'a> {
  type Item = Token;

  fn next(&mut self) -> Option<Self::Item> {
    self.inner.next_token()
  }
}

impl<'a> IntoIterator for Tokenizer<'a> {
  type Item = Token;

  type IntoIter = TokenizerIter<'a>;

  fn into_iter(self) -> Self::IntoIter {
    Self::IntoIter { inner: self }
  }
}

#[cfg(test)]
mod tests {
  use crate::internal::tokenizer::{Token, Tokenizer, TokenizerState};

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
        let mut tokenizer = Tokenizer::from_str(input);

        $(
          match tokenizer.next_token() {
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

        if let None = tokenizer.next_token() {
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
    for _ in Tokenizer::from_str("") {
      assert!(false, "Input is empty, tokenizer must've return None!");
    }

    assert!(true);
  }

  #[test]
  fn skip_whitespace() {
    let mut tokenizer = Tokenizer::from_str("     hello.       ");

    tokenizer.skip_whitespace();
    assert_eq!(Some(Token::Text(5, 10)), tokenizer.next_token());

    tokenizer.skip_whitespace();
    assert_eq!(None, tokenizer.next_token());
  }

  #[test]
  fn skip_but_no_whitespace() {
    let mut tokenizer = Tokenizer::from_str("hello.");

    tokenizer.skip_whitespace();
    assert_eq!(Some(Token::Text(0, 5)), tokenizer.next_token());
    assert_eq!(None, tokenizer.next_token());
  }

  #[test]
  fn skip_whitespace_empty() {
    let mut tokenizer = Tokenizer::from_str("");

    tokenizer.skip_whitespace();
    assert_eq!(None, tokenizer.next_token());
  }

  #[test]
  fn skip_while() {
    let text = r#""""""""text""#;
    let mut tokenizer = Tokenizer::from_str(text);

    tokenizer.skip_while(|tokens| match tokens {
      (Token::Text(_, _), Some(Token::DoubleQuote(_))) => false,
      _ => true,
    });

    assert_eq!(Some(Token::Text(7, 10)), tokenizer.next_token());
    assert_eq!(Some(Token::DoubleQuote(11)), tokenizer.next_token());
    assert_eq!(None, tokenizer.next_token());
  }

  #[test]
  fn take_while() {
    let text = r#"hello world""#;
    let mut tokenizer = Tokenizer::from_str(text);

    let tokens = tokenizer.take_while(|(token, _)| match token {
      Token::DoubleQuote(_) => false,
      _ => true,
    });
    assert_eq!(
      &[Token::Text(0, 4), Token::Whitespace(5), Token::Text(6, 10)],
      tokens.as_slice()
    );

    assert_eq!(Some(Token::DoubleQuote(11)), tokenizer.next_token());
    assert_eq!(None, tokenizer.next_token());
  }
}
