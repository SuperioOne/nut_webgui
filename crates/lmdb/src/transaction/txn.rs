use super::Transaction;
use crate::{
  DbHandle,
  error::{self, ErrorKind, LmdbError},
  ffi::{
    MDB_env, MDB_txn, MDB_val, mdb_get, mdb_put, mdb_txn_abort, mdb_txn_begin, mdb_txn_commit,
    mdb_txn_flags, mdb_txn_id, mdb_txn_prepare, result_fn,
  },
  flag::{TransactionFlag, WriteFlag},
  value::ValueRef,
};
use std::{ffi::c_uint, marker::PhantomData, ptr::null_mut};

pub struct Txn<'a> {
  handle: *mut MDB_txn,
  _phantom: PhantomData<&'a MDB_txn>,
}

impl<'a> Txn<'a> {
  pub fn new(env: *mut MDB_env, flags: TransactionFlag) -> Result<Self, LmdbError> {
    if env.is_null() {
      return Err(ErrorKind::NullHandle.into());
    }

    let mut txn: *mut MDB_txn = null_mut();
    result_fn!(mdb_txn_begin(env, null_mut(), flags.into_inner(), &mut txn))?;

    Ok(Self {
      handle: txn,
      _phantom: PhantomData,
    })
  }

  pub fn new_with_parent(
    env: *mut MDB_env,
    flags: TransactionFlag,
    parent: *mut MDB_txn,
  ) -> Result<Self, LmdbError> {
    if env.is_null() || parent.is_null() {
      return Err(ErrorKind::NullHandle.into());
    }

    let mut txn: *mut MDB_txn = null_mut();
    result_fn!(mdb_txn_begin(env, parent, flags.into_inner(), &mut txn))?;

    Ok(Self {
      handle: txn,
      _phantom: PhantomData,
    })
  }

  #[inline]
  pub const fn as_raw_ptr(&self) -> *mut MDB_txn {
    self.handle
  }

  pub fn get<'b>(
    &self,
    db_handle: DbHandle,
    key: &ValueRef<'b>,
  ) -> Result<Option<ValueRef<'_>>, LmdbError> {
    let key = key.as_mdb_val();
    let mut data = MDB_val {
      mv_size: 0,
      mv_data: null_mut(),
    };

    let result = result_fn!(mdb_get(
      self.as_raw_ptr(),
      db_handle,
      core::ptr::from_ref(key).cast_mut(),
      &mut data
    ));

    match result {
      Ok(()) => Ok(Some(ValueRef::from_mdb_val(data))),
      Err(err) => match err.kind() {
        ErrorKind::MdbError(error::MdbError::NotFound) => Ok(None),
        _ => Err(err),
      },
    }
  }

  pub fn put<'k, 'v>(
    &mut self,
    db_handle: DbHandle,
    key: &ValueRef<'k>,
    data: &ValueRef<'v>,
    flags: WriteFlag,
  ) -> Result<(), LmdbError> {
    let key = key.as_mdb_val();
    let data = data.as_mdb_val();

    result_fn!(mdb_put(
      self.handle,
      db_handle,
      core::ptr::from_ref(key).cast_mut(),
      core::ptr::from_ref(data).cast_mut(),
      flags.into_inner()
    ))
  }
}

impl<'a> Transaction for Txn<'a> {
  fn id(&self) -> usize {
    unsafe { mdb_txn_id(self.handle) }
  }

  fn prepare(&mut self) -> Result<(), LmdbError> {
    result_fn!(mdb_txn_prepare(self.handle))
  }

  fn commit(mut self) -> Result<(), LmdbError> {
    let result = result_fn!(mdb_txn_commit(self.handle));
    self.handle = null_mut();
    result
  }

  fn abort(mut self) {
    unsafe { mdb_txn_abort(self.handle) };
    self.handle = null_mut();
  }

  fn flags(&self) -> Result<TransactionFlag, LmdbError> {
    let mut flags: c_uint = 0;
    result_fn!(mdb_txn_flags(self.handle, &mut flags))?;
    Ok(TransactionFlag::from(flags))
  }
}

impl Drop for Txn<'_> {
  fn drop(&mut self) {
    if !self.handle.is_null() {
      unsafe { mdb_txn_abort(self.handle) }
    }
  }
}
