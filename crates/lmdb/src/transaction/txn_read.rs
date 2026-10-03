use super::{ChildTxn, Transaction, txn::Txn};
use crate::{
  DbHandle,
  env::{Env, ReadMarker, WriteMarker},
  error::LmdbError,
  flag::TransactionFlag,
  internal::{
    AsRawPtr,
    ffi::{MDB_txn, mdb_txn_env},
  },
  value::ValueRef,
};
use core::borrow::Borrow;

pub struct ReadTxn<'a> {
  inner: Txn<'a>,
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
  #[inline]
  pub(crate) fn new<R, W>(env: &'a Env<R, W>) -> Result<Self, LmdbError>
  where
    R: ReadMarker,
    W: WriteMarker,
  {
    Self::new_with_flags(env, TransactionFlag::new())
  }

  pub(crate) fn new_with_flags<R, W>(
    env: &'a Env<R, W>,
    mut flags: TransactionFlag,
  ) -> Result<Self, LmdbError>
  where
    R: ReadMarker,
    W: WriteMarker,
  {
    flags.set_assign(TransactionFlag::READONLY);
    let inner = Txn::new(env, flags)?;
    Ok(Self { inner })
  }

  #[inline]
  pub(super) fn from_txn(txn: Txn<'a>) -> Self {
    Self { inner: txn }
  }

  #[inline]
  pub fn get<'b, K>(&self, db_handle: DbHandle, key: K) -> Result<Option<ValueRef<'_>>, LmdbError>
  where
    K: Borrow<ValueRef<'b>>,
  {
    self.inner.get(db_handle, key.borrow())
  }

  pub fn begin_ro_child(&mut self) -> Result<ChildTxn<'_, Self>, LmdbError> {
    let env = unsafe { mdb_txn_env(self.inner.as_raw_ptr()) };
    let flags = self.flags()?.set(TransactionFlag::READONLY);
    let child = Txn::new_with_parent(env, flags, self.inner.as_raw_ptr())?;

    Ok(ChildTxn::from_child(Self::from_txn(child)))
  }
}

impl<'a> AsRawPtr for ReadTxn<'a> {
  type Return = MDB_txn;

  #[inline]
  fn as_raw_ptr(&self) -> *mut Self::Return {
    self.inner.as_raw_ptr()
  }
}
