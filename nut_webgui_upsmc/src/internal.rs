use self::lexer::Lexer;

pub mod ascii_rules;
pub mod escape;
pub mod item_pool;
pub mod lexer;
pub mod parser_utils;

// Inspired from http crates' Repr<T>
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Repr<S, C> {
  Standard(S),
  Custom(C),
}

pub trait Deserialize: Sized {
  type Error;

  fn deserialize(lexer: &mut Lexer) -> Result<Self, Self::Error>;
}

pub trait Serialize {
  type Output;

  fn serialize(self) -> Self::Output;
}
