#![allow(unused_assignments)]

#[cfg(test)]
mod test;

use crate::error::{Error, ErrorKind, ParseError};
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub enum Token {
  QuotedText {
    /// text starting index on target buffer
    start: usize,

    /// text ending index on target buffer.
    end: usize,
  },
  LF,
  Text {
    /// text starting index on target buffer
    start: usize,

    /// text ending index on target buffer.
    end: usize,
  },
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind {
  QuotedText,
  LF,
  Text,
}

impl Token {
  pub const fn kind(&self) -> TokenKind {
    match self {
      Token::QuotedText { .. } => TokenKind::QuotedText,
      Token::LF => TokenKind::LF,
      Token::Text { .. } => TokenKind::Text,
    }
  }
}

impl std::fmt::Display for TokenKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.as_str())
  }
}

impl TokenKind {
  pub const fn as_str(&self) -> &'static str {
    match self {
      TokenKind::QuotedText => "double-quoted text node",
      TokenKind::LF => "line feed node",
      TokenKind::Text => "text node",
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
  pub line: usize,
  pub col: usize,
}

impl Default for Position {
  fn default() -> Self {
    Self { line: 1, col: 1 }
  }
}

#[derive(Debug, Clone)]
struct State {
  read_head: usize,
  position: Position,
}

pub struct Lexer<'a> {
  buffer: &'a str,
  state: State,
}

impl<'a> Lexer<'a> {
  pub const fn new(buffer: &'a str) -> Self {
    Self {
      buffer,
      state: State {
        read_head: 0,
        position: Position { col: 1, line: 1 },
      },
    }
  }

  pub fn extract_from_token(&self, token: &Token) -> Cow<'_, str> {
    let buffer_str = &self.buffer;

    match token {
      Token::LF => Cow::Borrowed("\n"),
      Token::Text { start, end, .. } => Cow::Borrowed(&buffer_str[*start..*end]),
      Token::QuotedText { start, end, .. } => {
        let text = &buffer_str[*start..*end];
        let mut escaped = String::new();
        let mut slice_start: usize = 0;
        let mut text_iter = text.as_bytes().iter().enumerate();

        while let Some((idx, character)) = text_iter.next() {
          if *character == b'\\' {
            if slice_start < idx {
              escaped.push_str(&text[slice_start..idx]);
            }

            match text_iter.next() {
              Some((idx, c)) => {
                escaped.push(char::from(*c));
                slice_start = idx + 1;
              }
              None => break,
            }
          }
        }

        // If escaped is still empty, it means it didn't escaped anything.
        if escaped.is_empty() {
          Cow::Borrowed(&buffer_str[*start..*end])
        } else {
          if slice_start < text.len() {
            escaped.push_str(&text[slice_start..]);
          }

          Cow::Owned(escaped)
        }
      }
    }
  }

  pub fn peek(&self) -> Option<Token> {
    let mut tmp_tokenizer = Self {
      state: self.state.clone(),
      buffer: self.buffer,
    };

    tmp_tokenizer.next_token().unwrap_or_default()
  }

  pub fn peek_as_str(&self) -> Option<&str> {
    match self.peek() {
      Some(Token::LF) => Some("\n"),
      Some(Token::Text { start, end }) => Some(&self.buffer[start..end]),
      Some(Token::QuotedText { start, end }) => Some(&self.buffer[start..end]),
      None => None,
    }
  }

  #[inline]
  pub const fn get_positon(&self) -> Position {
    self.state.position
  }

  #[inline]
  fn read_text(&mut self) -> (usize, usize) {
    let start = self.state.read_head;
    let mut end = start;
    let buffer = &self.buffer.as_bytes()[start..];

    for char_byte in buffer {
      match *char_byte {
        v if v.is_ascii_whitespace() => {
          break;
        }
        _ => {
          self.move_read_head(1);
          end = self.state.read_head;
        }
      }
    }

    (start, end)
  }

  #[inline]
  fn read_quoted_text(&mut self) -> Result<(usize, usize), Error> {
    let start = self.state.read_head;
    let mut end = start;
    let mut buffer_iter = self.buffer.as_bytes()[self.state.read_head..].iter();

    loop {
      let char = buffer_iter.next();

      match char {
        Some(b'\\') => {
          // Escape next character
          _ = buffer_iter.next();
          self.move_read_head(2);
        }
        Some(b'"') => {
          end = self.state.read_head;
          break;
        }
        Some(_) => {
          self.move_read_head(1);
        }
        None => {
          return Err(
            ErrorKind::ParseError {
              inner: ParseError::ExpectedDoubleQuote,
              position: self.state.position,
            }
            .into(),
          );
        }
      }
    }

    Ok((start, end))
  }

  #[inline]
  const fn move_read_head(&mut self, by: isize) {
    self.state.read_head = self.state.read_head.saturating_add_signed(by);
    self.state.position.col = self.state.position.col.saturating_add_signed(by);
  }

  pub fn skip_whitespaces(&mut self) {
    let buffer = &self.buffer.as_bytes()[self.state.read_head..];

    for char_byte in buffer.iter() {
      match char_byte {
        b'\n' => {
          self.state.read_head += 1;
          self.state.position.col = 1;
          self.state.position.line += 1;
        }
        v if v.is_ascii_whitespace() => {
          self.move_read_head(1);
        }
        _ => break,
      }
    }
  }

  #[inline]
  pub fn is_finished(&self) -> bool {
    self.peek().is_none()
  }

  pub fn next_token(&mut self) -> Result<Option<Token>, Error> {
    let buffer = &self.buffer.as_bytes()[self.state.read_head..];

    for char_byte in buffer.iter() {
      match char_byte {
        b'\n' => {
          let token = Some(Token::LF);

          self.state.read_head += 1;
          self.state.position.col = 0;
          self.state.position.line += 1;

          return Ok(token);
        }
        v if v.is_ascii_whitespace() => {
          self.move_read_head(1);
        }
        b'"' => {
          self.move_read_head(1);
          let (start, end) = self.read_quoted_text()?;
          self.move_read_head(1);

          return Ok(Some(Token::QuotedText { start, end }));
        }
        _ => {
          let (start, end) = self.read_text();
          return Ok(Some(Token::Text { start, end }));
        }
      }
    }

    // Buffer is empty
    Ok(None)
  }
}
