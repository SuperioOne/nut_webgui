use super::{Env, ReadMarker, WriteMarker};
use crate::{
  DbHandle, StatInfo,
  env::path_to_cstr,
  error::LmdbError,
  flag::{CopyFlag, DbFlag, TransactionFlag},
  internal::{
    AsRawPtr as _,
    ffi::{
      mdb_dbi_flags, mdb_env_copy2, mdb_env_copyfd2, mdb_env_incr_dump, mdb_env_incr_dumpfd,
      mdb_stat, result_fn,
    },
  },
  transaction::{ReadTxn, Transaction as _},
};
use std::{fs::File, os::fd::AsRawFd as _, path::Path};

/// Reader with no thread local data.
#[derive(Clone, Copy)]
pub struct MdbReader;

impl ReadMarker for MdbReader {}

impl<W> Env<MdbReader, W>
where
  W: WriteMarker,
{
  pub fn begin_ro_transaction(&self) -> Result<ReadTxn<'_>, LmdbError> {
    ReadTxn::new(self)
  }

  pub fn begin_ro_transaction_with_flags(
    &self,
    flags: TransactionFlag,
  ) -> Result<ReadTxn<'_>, LmdbError> {
    ReadTxn::new_with_flags(self, flags)
  }

  pub fn db_flags(&self, db_handle: DbHandle) -> Result<DbFlag, LmdbError> {
    let mut value: u32 = 0;
    let txn = ReadTxn::new(self)?;

    result_fn!(mdb_dbi_flags(txn.as_raw_ptr(), db_handle, &mut value))?;
    txn.commit()?;

    Ok(DbFlag::from(value))
  }

  pub fn db_stats(&self, db_handle: DbHandle) -> Result<StatInfo, LmdbError> {
    let mut stats = StatInfo::default();
    let txn = ReadTxn::new(self)?;

    result_fn!(mdb_stat(txn.as_raw_ptr(), db_handle, &mut stats))?;
    txn.commit()?;

    Ok(stats)
  }

  pub fn incr_dump_to_path<P>(&self, path: P, txnid: usize) -> Result<(), LmdbError>
  where
    P: AsRef<Path>,
  {
    let path = path_to_cstr(path)?;
    result_fn!(mdb_env_incr_dump(self.handle, path.as_ptr(), txnid))
  }

  pub fn incr_dump_to_file(&self, file: &mut File, txnid: usize) -> Result<(), LmdbError> {
    result_fn!(mdb_env_incr_dumpfd(self.handle, file.as_raw_fd(), txnid))
  }

  pub fn copy_to_dir<P>(&self, path: P, flags: CopyFlag) -> Result<(), LmdbError>
  where
    P: AsRef<Path>,
  {
    let path = path_to_cstr(path)?;
    result_fn!(mdb_env_copy2(
      self.handle,
      path.as_ptr(),
      flags.into_inner()
    ))
  }

  pub fn copy_to_file(&self, file: &mut File, flags: CopyFlag) -> Result<(), LmdbError> {
    result_fn!(mdb_env_copyfd2(
      self.handle,
      file.as_raw_fd(),
      flags.into_inner()
    ))
  }
}
