use crate::{
  DbHandle, StatInfo,
  env::Env,
  error::LmdbError,
  ffi::{mdb_dbi_flags, mdb_stat, result_fn},
  flag::{DbFlag, TransactionFlag},
  transaction::{ReadTxn, Transaction, WriteTxn, txn::Txn},
};

pub struct Database<'a> {
  db_handle: DbHandle,
  env: &'a Env,
}

impl<'a> Database<'a> {
  #[inline]
  pub(crate) const fn new(env: &'a Env, db_handle: DbHandle) -> Self {
    Self { db_handle, env }
  }

  pub fn flags(&self) -> Result<DbFlag, LmdbError> {
    let mut value: u32 = 0;
    let txn = Txn::new(self.env.as_raw_ptr(), TransactionFlag::READONLY)?;

    result_fn!(mdb_dbi_flags(txn.as_raw_ptr(), self.db_handle, &mut value))?;
    txn.commit()?;

    Ok(DbFlag::from(value))
  }

  pub fn stats(&self) -> Result<StatInfo, LmdbError> {
    let mut stats = StatInfo::default();
    let txn = Txn::new(self.env.as_raw_ptr(), TransactionFlag::READONLY)?;

    result_fn!(mdb_stat(txn.as_raw_ptr(), self.db_handle, &mut stats))?;
    txn.commit()?;

    Ok(stats)
  }

  pub fn begin_ro_transaction(&mut self) -> Result<ReadTxn<'_>, LmdbError> {
    let txn = ReadTxn::new(self.env.as_raw_ptr(), self.db_handle)?;
    Ok(txn)
  }

  pub fn begin_rw_transaction(&mut self) -> Result<WriteTxn<'_>, LmdbError> {
    let txn = WriteTxn::new(self.env.as_raw_ptr(), self.db_handle)?;
    Ok(txn)
  }
}
