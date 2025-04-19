use self::lexer::Lexer;

pub mod ascii_rules;
pub mod lexer;
pub mod parser_utils;

// Inspired from http crates' Repr<T>
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Repr<S, C> {
  Standard(S),
  Custom(C),
}

/// Alias for [`Box<str>`]
pub type ReadOnlyStr = Box<str>;

pub trait DeserializeResponse: Sized {
  type Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error>;
}
