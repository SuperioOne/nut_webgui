use super::{
  error::{ErrorKind, LmdbError},
  ffi::{
    MDB_env, mdb_env_create, mdb_env_incr_dumpfd, mdb_env_open, mdb_env_set_mapsize,
    mdb_env_set_maxdbs, mdb_env_set_maxreaders, mdb_env_set_pagesize, mdb_mode_t, result_fn,
  },
  flag::{CopyFlag, EnvFlag},
};
use crate::{
  DbHandle, EnvInfo, StatInfo,
  database::Database,
  db_name::DbName,
  ffi::{
    mdb_dbi_close, mdb_dbi_open, mdb_drop, mdb_env_close, mdb_env_copy2, mdb_env_copyfd2,
    mdb_env_incr_dump, mdb_env_incr_loadfd, mdb_env_info, mdb_env_rollback, mdb_env_set_flags,
    mdb_env_stat,
  },
  flag::{DbFlag, TransactionFlag},
  transaction::{Transaction, txn::Txn},
};
use core::ptr::null_mut;
use std::{
  borrow::Borrow,
  collections::BTreeMap,
  ffi::CString,
  fs::File,
  num::{NonZeroU32, NonZeroUsize},
  os::fd::AsRawFd,
  path::Path,
  str::FromStr,
};

pub struct EnvBuilder {
  flags: EnvFlag,
  map_size: Option<NonZeroUsize>,
  max_dbs: Option<NonZeroU32>,
  max_readers: Option<NonZeroU32>,
  page_size: Option<NonZeroUsize>,
  permissions: mdb_mode_t,
}

pub struct Env {
  handle: *mut MDB_env,
  open_dbs: BTreeMap<DbName, DbHandle>,
}

#[inline]
fn path_to_cstr<P>(path: P) -> Result<CString, LmdbError>
where
  P: AsRef<Path>,
{
  let path_str = path.as_ref().to_str().ok_or(ErrorKind::PathError)?;
  let path = CString::from_str(path_str).map_err(|_| ErrorKind::PathError)?;
  Ok(path)
}

impl Env {
  #[inline]
  pub const fn new() -> EnvBuilder {
    EnvBuilder::new()
  }

  fn init() -> Result<Self, LmdbError> {
    let mut env = Self {
      handle: null_mut(),
      open_dbs: BTreeMap::new(),
    };
    result_fn!(mdb_env_create(&mut env.handle))?;
    Ok(env)
  }

  fn set_map_size(&mut self, size: NonZeroUsize) -> Result<(), LmdbError> {
    result_fn!(mdb_env_set_mapsize(self.handle, size.get()))?;
    Ok(())
  }

  fn set_page_size(&mut self, size: NonZeroUsize) -> Result<(), LmdbError> {
    result_fn!(mdb_env_set_pagesize(self.handle, size.get() as i32))?;
    Ok(())
  }

  fn set_max_dbs(&mut self, max: NonZeroU32) -> Result<(), LmdbError> {
    result_fn!(mdb_env_set_maxdbs(self.handle, max.get()))?;
    Ok(())
  }

  fn set_max_readers(&mut self, max: NonZeroU32) -> Result<(), LmdbError> {
    result_fn!(mdb_env_set_maxreaders(self.handle, max.get()))?;
    Ok(())
  }

  fn set_flags(&mut self, flags: EnvFlag) -> Result<(), LmdbError> {
    result_fn!(mdb_env_set_flags(self.handle, flags.into_inner(), 1))
  }

  fn open(&mut self, path: &Path, permissions: mdb_mode_t) -> Result<(), LmdbError> {
    let path = path_to_cstr(path)?;
    result_fn!(mdb_env_open(self.handle, path.as_ptr(), 0, permissions))?;
    Ok(())
  }

  #[inline]
  pub(crate) const fn as_raw_ptr(&self) -> *mut MDB_env {
    self.handle
  }

  /// Open multiple database in a single transaction
  pub fn open_databases<'a, I>(&mut self, names: I, flags: DbFlag) -> Result<(), LmdbError>
  where
    I: Iterator<Item = &'a DbName>,
  {
    let txn = Txn::new(self.handle, TransactionFlag::new())?;

    for name in names {
      if !self.open_dbs.contains_key(name) {
        let mut handle: DbHandle = 0;

        result_fn!(mdb_dbi_open(
          txn.as_raw_ptr(),
          name.as_ptr(),
          flags.into_inner(),
          &mut handle
        ))?;

        _ = self.open_dbs.insert(name.clone(), handle);
      }
    }

    txn.commit()?;
    Ok(())
  }

  /// Open database and return it's handle.
  pub fn open_database<D>(&mut self, name: D, flags: DbFlag) -> Result<Database<'_>, LmdbError>
  where
    D: Borrow<DbName>,
  {
    let key = name.borrow();
    match self.open_dbs.get(key) {
      Some(handle) => Ok(Database::new(self, *handle)),
      None => {
        let txn = Txn::new(self.handle, TransactionFlag::new())?;
        let mut handle: DbHandle = 0;

        result_fn!(mdb_dbi_open(
          txn.as_raw_ptr(),
          name.borrow().as_ptr(),
          flags.into_inner(),
          &mut handle
        ))?;

        txn.commit()?;

        _ = self.open_dbs.insert(key.clone(), handle);
        Ok(Database::new(self, handle))
      }
    }
  }

  /// Get already opened database handle
  pub fn get_database<D>(&self, name: D) -> Result<Database<'_>, LmdbError>
  where
    D: Borrow<DbName>,
  {
    match self.open_dbs.get(name.borrow()) {
      Some(handle) => Ok(Database::new(self, *handle)),
      None => Err(ErrorKind::DbNotOpen.into()),
    }
  }

  pub fn close_database<D>(&mut self, name: D) -> Result<(), LmdbError>
  where
    D: Borrow<DbName>,
  {
    match self.open_dbs.remove(name.borrow()) {
      Some(handle) => {
        unsafe { mdb_dbi_close(self.handle, handle) };
        Ok(())
      }
      None => Err(ErrorKind::DbNotOpen.into()),
    }
  }

  pub fn delete_database<D>(&mut self, name: D) -> Result<(), LmdbError>
  where
    D: Borrow<DbName>,
  {
    match self.open_dbs.get(name.borrow()) {
      Some(handle) => {
        let txn = Txn::new(self.handle, TransactionFlag::new())?;

        match result_fn!(mdb_drop(txn.as_raw_ptr(), *handle, 1)) {
          Ok(_) => {
            self.open_dbs.remove(name.borrow());
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

  pub fn empty_database<D>(&mut self, name: D) -> Result<(), LmdbError>
  where
    D: Borrow<DbName>,
  {
    match self.open_dbs.get(name.borrow()) {
      Some(handle) => {
        let txn = Txn::new(self.handle, TransactionFlag::new())?;

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

  pub fn rollback_transaction(&self, txnid: usize) -> Result<(), LmdbError> {
    result_fn!(mdb_env_rollback(self.handle, txnid))
  }

  pub fn info(&self) -> Result<EnvInfo, LmdbError> {
    let mut info = EnvInfo::default();
    result_fn!(mdb_env_info(self.handle, &mut info))?;
    Ok(info)
  }

  pub fn stats(&self) -> Result<StatInfo, LmdbError> {
    let mut stat = StatInfo::default();
    result_fn!(mdb_env_stat(self.handle, &mut stat))?;
    Ok(stat)
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

  pub fn load_dump_from_file(&mut self, file: &File) -> Result<(), LmdbError> {
    result_fn!(mdb_env_incr_loadfd(self.handle, file.as_raw_fd()))
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

  #[inline]
  pub fn close(self) {
    drop(self)
  }
}

impl EnvBuilder {
  pub const fn new() -> Self {
    Self {
      flags: EnvFlag::new(),
      map_size: None,
      max_dbs: None,
      max_readers: None,
      page_size: None,
      permissions: 0o644,
    }
  }

  #[inline]
  pub const fn set_max_dbs(mut self, value: NonZeroU32) -> Self {
    self.max_dbs = Some(value);
    self
  }

  #[inline]
  pub const fn set_max_readers(mut self, value: NonZeroU32) -> Self {
    self.max_readers = Some(value);
    self
  }

  #[inline]
  pub const fn set_page_size(mut self, value: NonZeroUsize) -> Self {
    self.page_size = Some(value);
    self
  }

  #[inline]
  pub const fn set_map_size(mut self, value: NonZeroUsize) -> Self {
    self.map_size = Some(value);
    self
  }

  #[inline]
  pub const fn set_flags(mut self, value: EnvFlag) -> Self {
    self.flags = value;
    self
  }

  #[inline]
  pub const fn set_permissions(mut self, value: u32) -> Self {
    self.permissions = value as mdb_mode_t;
    self
  }

  pub fn open<P>(self, path: P) -> Result<Env, LmdbError>
  where
    P: AsRef<Path>,
  {
    let mut env = Env::init()?;

    if let Some(val) = self.map_size {
      env.set_map_size(val)?;
    }

    if let Some(val) = self.page_size {
      env.set_page_size(val)?;
    }

    if let Some(val) = self.max_readers {
      env.set_max_readers(val)?;
    }

    if let Some(val) = self.max_dbs {
      env.set_max_dbs(val)?;
    }

    if !self.flags.is_empty() {
      env.set_flags(self.flags)?;
    }

    env.open(path.as_ref(), self.permissions)?;

    Ok(env)
  }
}

unsafe impl Send for Env {}
unsafe impl Sync for Env {}

impl Drop for Env {
  fn drop(&mut self) {
    if !self.handle.is_null() {
      for (_, dbi) in self.open_dbs.iter() {
        unsafe {
          mdb_dbi_close(self.handle, *dbi);
        }
      }

      unsafe { mdb_env_close(self.handle) };
    }
  }
}
