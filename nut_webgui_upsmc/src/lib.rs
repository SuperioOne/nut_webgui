pub mod client;
pub mod errors;
pub mod parser;
pub mod ups_variables;

#[derive(Debug)]
pub struct Ups {
  pub name: Box<str>,
  pub desc: Box<str>,
}
