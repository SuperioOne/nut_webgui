use super::{Transaction, txn::Txn};
use crate::{error::LmdbError, flag::TransactionFlag};
use std::{
  marker::PhantomData,
  ops::{Deref, DerefMut},
};

pub struct ChildTxn<'a, C>
where
  C: Transaction,
{
  handle: C,
  _phantom: PhantomData<&'a mut Txn<'a>>,
}

impl<'a, C> ChildTxn<'a, C>
where
  C: Transaction,
{
  #[inline]
  pub const fn from_child(txn: C) -> Self {
    Self {
      handle: txn,
      _phantom: PhantomData,
    }
  }
}

impl<'a, C> Transaction for ChildTxn<'a, C>
where
  C: Transaction,
{
  #[inline]
  fn id(&self) -> usize {
    self.handle.id()
  }

  #[inline]
  fn prepare(&mut self) -> Result<(), LmdbError> {
    self.handle.prepare()
  }

  fn commit(self) -> Result<(), LmdbError> {
    self.handle.commit()
  }

  fn abort(self) {
    self.handle.abort();
  }

  #[inline]
  fn flags(&self) -> Result<TransactionFlag, LmdbError> {
    self.handle.flags()
  }
}

impl<'a, C> Deref for ChildTxn<'a, C>
where
  C: Transaction,
{
  type Target = C;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.handle
  }
}

impl<'a, C> DerefMut for ChildTxn<'a, C>
where
  C: Transaction,
{
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.handle
  }
}
