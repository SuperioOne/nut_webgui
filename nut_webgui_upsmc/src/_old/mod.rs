mod client;
pub mod parser;
pub mod ups_variables;

#[derive(Debug)]
pub struct Ups {
  pub name: Box<str>,
  pub desc: Box<str>,
}

pub use client::*;
