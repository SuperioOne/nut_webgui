use super::{ChildTxn, ReadTxn, Transaction, txn::Txn};
use crate::{
  DbHandle,
  env::{Env, ReadMarker, WriteMarker},
  error::LmdbError,
  flag::{TransactionFlag, WriteFlag},
  internal::{
    AsRawPtr,
    ffi::{MDB_txn, mdb_txn_env},
  },
  value::ValueRef,
};
use core::borrow::Borrow;

pub struct WriteTxn<'a> {
  inner: Txn<'a>,
}

impl Transaction for WriteTxn<'_> {
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

impl<'a> WriteTxn<'a> {
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
    flags.unset_assign(TransactionFlag::READONLY);
    let inner = Txn::new(env, flags)?;
    Ok(Self { inner })
  }

  #[inline]
  pub(super) fn from_txn(txn: Txn<'a>) -> Self {
    Self { inner: txn }
  }

  pub fn begin_ro_child(&mut self) -> Result<ChildTxn<'_, ReadTxn<'a>>, LmdbError> {
    let env = unsafe { mdb_txn_env(self.inner.as_raw_ptr()) };
    let flags = self.flags()?.set(TransactionFlag::READONLY);
    let child = Txn::new_with_parent(env, flags, self.inner.as_raw_ptr())?;

    Ok(ChildTxn::from_child(ReadTxn::from_txn(child)))
  }

  pub fn begin_rw_child(&mut self) -> Result<ChildTxn<'_, Self>, LmdbError> {
    let env = unsafe { mdb_txn_env(self.inner.as_raw_ptr()) };
    let flags = self.flags()?.unset(TransactionFlag::READONLY);
    let child = Txn::new_with_parent(env, flags, self.inner.as_raw_ptr())?;

    Ok(ChildTxn::from_child(Self::from_txn(child)))
  }

  pub fn get<'b, K>(&self, db_handle: DbHandle, key: K) -> Result<Option<ValueRef<'_>>, LmdbError>
  where
    K: Borrow<ValueRef<'b>>,
  {
    self.inner.get(db_handle, key.borrow())
  }

  pub fn put<'k, 'v, K, V>(
    &mut self,
    db_handle: DbHandle,
    key: K,
    data: V,
    flags: WriteFlag,
  ) -> Result<(), LmdbError>
  where
    K: Borrow<ValueRef<'k>>,
    V: Borrow<ValueRef<'v>>,
  {
    self
      .inner
      .put(db_handle, key.borrow(), data.borrow(), flags)
  }
}

impl<'a> AsRawPtr for WriteTxn<'a> {
  type Return = MDB_txn;

  #[inline]
  fn as_raw_ptr(&self) -> *mut Self::Return {
    self.inner.as_raw_ptr()
  }
}
