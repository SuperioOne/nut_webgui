use super::error::LmdbError;
use crate::flag::TransactionFlag;

pub(crate) mod txn;
mod txn_child;
mod txn_read;
mod txn_write;

pub use txn_child::*;
pub use txn_read::*;
pub use txn_write::*;

pub trait Transaction {
  fn id(&self) -> usize;

  fn prepare(&mut self) -> Result<(), LmdbError>;

  fn commit(self) -> Result<(), LmdbError>;

  fn abort(self);

  fn flags(&self) -> Result<TransactionFlag, LmdbError>;
}
