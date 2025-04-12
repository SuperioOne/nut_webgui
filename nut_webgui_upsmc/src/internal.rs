use self::lexer::Lexer;

pub mod ascii_rules;
pub mod lexer;

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

macro_rules! gen_parser {
  (
   $(( $($tokens:tt)+ ));+
  ) => {

    $(
        /// line
       $(let $tokens = "";)+
     )+
  }
}

macro_rules! parser_syntax {
    ($token:literal $($rest:tt)*) => {
      let a = $token;
      parser_syntax!($($rest)*);
    };

    ({UPS$(, name = $name:ident)?} $($rest:tt)+) => {
      $(let $name = 0;)?
      parser_syntax!($($rest)*);
    };

    ({VAR$(, name = $name:ident)?} $($rest:tt)*) => {
      let var_name = 0;
      parser_syntax!($($rest)*);
    };

    ({CMD$(, name = $name:ident)?} $($rest:tt)*) => {
      let cmd_name = 0;
      parser_syntax!($($rest)*);
    };

    ({VALUE$(, name = $name:ident)?} $($rest:tt)*) => {
      let value_var = 0;
      parser_syntax!($($rest)*);
    };

    () => {

    }
}

pub(crate) use gen_parser;
pub(crate) use parser_syntax;

fn test() {
  gen_parser!((test test5 test2 test4); (test6 line2));
  // parser_syntax!({UPS, name = test} "literal" {CMD} {VALUE} "test" {VAR});
  //
  // ("BEGIN" "LIST" "VAR" {UPS});
  // ("VAR" {UPS} {VAR} {VALUE});
  // ("END" "LIST" "VAR" {UPS});
}
