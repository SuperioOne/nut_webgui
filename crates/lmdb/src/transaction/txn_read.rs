use super::{ChildTxn, Transaction, txn::Txn};
use crate::{
  DbHandle,
  error::LmdbError,
  ffi::{MDB_env, mdb_txn_env},
  flag::TransactionFlag,
  value::ValueRef,
};
use std::borrow::Borrow;

pub struct ReadTxn<'a> {
  inner: Txn<'a>,
  db_handle: DbHandle,
}

impl Transaction for ReadTxn<'_> {
  #[inline]
  fn id(&self) -> usize {
    self.inner.id()
  }

  #[inline]
  fn prepare(&mut self) -> Result<(), LmdbError> {
    self.inner.prepare()
  }

  #[inline]
  fn commit(self) -> Result<(), LmdbError> {
    self.inner.commit()
  }

  #[inline]
  fn abort(self) {
    self.inner.abort()
  }

  #[inline]
  fn flags(&self) -> Result<TransactionFlag, LmdbError> {
    self.inner.flags()
  }
}

impl<'a> ReadTxn<'a> {
  pub(crate) fn new(env: *mut MDB_env, dbi_handle: DbHandle) -> Result<Self, LmdbError> {
    let inner = Txn::new(env, TransactionFlag::READONLY)?;
    Ok(Self {
      inner,
      db_handle: dbi_handle,
    })
  }

  #[inline]
  pub(super) fn from_txn(txn: Txn<'a>, db_handle: DbHandle) -> Self {
    Self {
      inner: txn,
      db_handle,
    }
  }

  #[inline]
  pub fn get<'b, K>(&self, key: K) -> Result<Option<ValueRef<'_>>, LmdbError>
  where
    K: Borrow<ValueRef<'b>>,
  {
    self.inner.get(self.db_handle, key.borrow())
  }

  pub fn begin_ro_child(&mut self) -> Result<ChildTxn<'_, Self>, LmdbError> {
    let env = unsafe { mdb_txn_env(self.inner.as_raw_ptr()) };
    let flags = self.flags()?.set(TransactionFlag::READONLY);
    let child = Txn::new_with_parent(env, flags, self.inner.as_raw_ptr())?;

    Ok(ChildTxn::from_child(Self::from_txn(child, self.db_handle)))
  }
}
