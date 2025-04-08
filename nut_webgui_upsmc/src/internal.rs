pub mod ascii_rules;
pub mod tokenizer;
pub mod word_split;

// Inspired from http crates' Repr<T>
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Repr<S, C> {
  Standard(S),
  Custom(C),
}

/// Alias for [`Box<str>`]
pub type ReadOnlyStr = Box<str>;
