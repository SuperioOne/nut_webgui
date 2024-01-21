use crate::upsd_client::protocol::UpsVariable;

pub mod client;
pub mod errors;
mod parser;
pub mod protocol;

#[derive(Debug)]
pub struct Ups {
  pub name: Box<str>,
  pub desc: Box<str>,
}

#[derive(Debug)]
pub struct Var {
  pub name: Box<str>,
  pub var: UpsVariable,
}

#[derive(Debug)]
pub struct Cmd {
  pub name: Box<str>,
  pub cmd: Box<str>,
}
