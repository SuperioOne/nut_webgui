use super::{Env, ReadMarker, WriteMarker};
use crate::{
  DbHandle,
  db_name::DbName,
  error::{ErrorKind, LmdbError},
  flag::{DbFlag, TransactionFlag},
  internal::{
    AsRawPtr as _,
    ffi::{mdb_dbi_open, mdb_drop, mdb_env_incr_loadfd, mdb_env_rollback, result_fn},
  },
  transaction::{Transaction as _, WriteTxn},
};
use std::{borrow::Borrow, fs::File, os::fd::AsRawFd as _};

/// Primary type for write support.
#[derive(Clone, Copy)]
pub struct MdbWriter;

/// Empty placeholder type for readonly environments.
#[derive(Clone, Copy)]
pub struct MdbEmptyWriter;

impl WriteMarker for MdbWriter {}
impl WriteMarker for MdbEmptyWriter {}

impl<R> Env<R, MdbWriter>
where
  R: ReadMarker,
{
  #[inline]
  pub fn begin_rw_transaction(&mut self) -> Result<WriteTxn<'_>, LmdbError> {
    WriteTxn::new(self)
  }

  pub fn begin_rw_transaction_with_flags(
    &mut self,
    flags: TransactionFlag,
  ) -> Result<WriteTxn<'_>, LmdbError> {
    WriteTxn::new_with_flags(self, flags)
  }

  /// Create and open multiple database in a single transaction
  pub fn create_databases<'a, I>(&mut self, names: I, mut flags: DbFlag) -> Result<(), LmdbError>
  where
    I: Iterator<Item = &'a DbName>,
  {
    let txn = WriteTxn::new(self)?;
    let mut open_dbs = Vec::new();
    flags.set_assign(DbFlag::CREATE);

    for name in names {
      if !self.open_dbs.contains_key(name) {
        let mut handle: DbHandle = 0;

        result_fn!(mdb_dbi_open(
          txn.as_raw_ptr(),
          name.as_ptr(),
          flags.into_inner(),
          &mut handle
        ))?;

        open_dbs.push((handle, name));
      }
    }

    txn.commit()?;

    for (handle, name) in open_dbs.into_iter() {
      _ = self.open_dbs.insert(name.clone(), handle);
    }

    Ok(())
  }

  /// Open database.
  pub fn create_database<D>(&mut self, name: D, mut flags: DbFlag) -> Result<DbHandle, LmdbError>
  where
    D: Borrow<DbName>,
  {
    let key = name.borrow();
    flags.set_assign(DbFlag::CREATE);

    match self.open_dbs.get(key) {
      Some(handle) => Ok(*handle),
      None => {
        let txn = WriteTxn::new(self)?;
        let mut handle: DbHandle = 0;

        result_fn!(mdb_dbi_open(
          txn.as_raw_ptr(),
          name.borrow().as_ptr(),
          flags.into_inner(),
          &mut handle
        ))?;

        txn.commit()?;

        _ = self.open_dbs.insert(key.clone(), handle);
        Ok(handle)
      }
    }
  }

  pub fn drop_database<D>(&mut self, name: D) -> Result<(), LmdbError>
  where
    D: Borrow<DbName>,
  {
    match self.open_dbs.get(name.borrow()) {
      Some(handle) => {
        let txn = WriteTxn::new(self)?;

        match result_fn!(mdb_drop(txn.as_raw_ptr(), *handle, 1)) {
          Ok(_) => {
            txn.commit()?;
            self.open_dbs.remove(name.borrow());
            Ok(())
          }
          Err(err) => {
            txn.abort();
            Err(err)
          }
        }
      }
      None => Err(ErrorKind::DbNotOpen.into()),
    }
  }

  pub fn empty_database<D>(&mut self, name: D) -> Result<(), LmdbError>
  where
    D: Borrow<DbName>,
  {
    match self.open_dbs.get(name.borrow()) {
      Some(handle) => {
        let txn = WriteTxn::new(self)?;

        match result_fn!(mdb_drop(txn.as_raw_ptr(), *handle, 0)) {
          Ok(_) => {
            txn.commit()?;
            Ok(())
          }
          Err(err) => {
            txn.abort();
            Err(err)
          }
        }
      }
      None => Err(ErrorKind::DbNotOpen.into()),
    }
  }

  pub fn rollback_transaction(&mut self, txnid: usize) -> Result<(), LmdbError> {
    result_fn!(mdb_env_rollback(self.handle, txnid))
  }

  pub fn load_dump_from_file(&mut self, file: &File) -> Result<(), LmdbError> {
    result_fn!(mdb_env_incr_loadfd(self.handle, file.as_raw_fd()))
  }
}
