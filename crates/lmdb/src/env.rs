use self::{builder::Builder, env_reader::MdbReader, env_writer::MdbEmptyWriter};
use crate::{
  DbHandle, EnvInfo, StatInfo,
  db_name::DbName,
  error::{ErrorKind, LmdbError},
  flag::{DbFlag, EnvFlag},
  internal::{
    AsRawPtr,
    ffi::{
      MDB_env, mdb_dbi_close, mdb_dbi_open, mdb_env_close, mdb_env_create, mdb_env_info,
      mdb_env_open, mdb_env_set_mapsize, mdb_env_set_maxdbs, mdb_env_set_maxreaders,
      mdb_env_set_pagesize, mdb_env_stat, mdb_mode_t, mdb_reader_check, mdb_reader_list, result_fn,
    },
  },
  reader_info::{ReaderInfo, parse_reader_list},
  transaction::{ReadTxn, Transaction},
};
use core::{
  borrow::Borrow,
  marker::PhantomData,
  num::{NonZeroU32, NonZeroUsize},
  ptr::null_mut,
};
use std::{
  collections::BTreeMap,
  ffi::{CStr, CString, c_char, c_int, c_void},
  os::unix::ffi::OsStrExt,
  path::Path,
};

pub mod builder;
pub mod env_reader;
pub mod env_tls_reader;
pub mod env_writer;

/// Marker trait for write component
pub trait WriteMarker {}

/// Marker trait for read component
pub trait ReadMarker {}

/// Default environment opener.
pub type EnvBuilder = Builder<MdbReader, MdbEmptyWriter>;

pub struct Env<R, W>
where
  R: ReadMarker,
  W: WriteMarker,
{
  handle: *mut MDB_env,
  open_dbs: BTreeMap<DbName, DbHandle>,
  _phantom_r: PhantomData<R>,
  _phantom_w: PhantomData<W>,
}

fn path_to_cstr<P>(path: P) -> Result<CString, LmdbError>
where
  P: AsRef<Path>,
{
  let path_bytes = path.as_ref().as_os_str().as_bytes();
  let path = CString::new(path_bytes).map_err(|_| ErrorKind::PathError)?;
  Ok(path)
}

impl<R, W> Env<R, W>
where
  R: ReadMarker,
  W: WriteMarker,
{
  fn init() -> Result<Self, LmdbError> {
    let mut env = Self {
      handle: null_mut(),
      open_dbs: BTreeMap::new(),
      _phantom_r: PhantomData,
      _phantom_w: PhantomData,
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

  fn open(
    &mut self,
    path: &Path,
    flags: EnvFlag,
    permissions: mdb_mode_t,
  ) -> Result<(), LmdbError> {
    let path = path_to_cstr(path)?;

    result_fn!(mdb_env_open(
      self.handle,
      path.as_ptr(),
      flags.into_inner(),
      permissions
    ))?;

    Ok(())
  }

  /// Open multiple database in a single transaction
  pub fn open_databases<'a, I>(&mut self, names: I, mut flags: DbFlag) -> Result<(), LmdbError>
  where
    I: Iterator<Item = &'a DbName>,
  {
    let txn = ReadTxn::new(self)?;
    let mut open_dbs = Vec::new();
    flags.unset_assign(DbFlag::CREATE);

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
  pub fn open_database<D>(&mut self, name: D, mut flags: DbFlag) -> Result<DbHandle, LmdbError>
  where
    D: Borrow<DbName>,
  {
    let key = name.borrow();
    flags.unset_assign(DbFlag::CREATE);

    match self.open_dbs.get(key) {
      Some(handle) => Ok(*handle),
      None => {
        let txn = ReadTxn::new(self)?;
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

  /// Get already opened database handle
  pub fn get_open_database<D>(&self, name: D) -> Result<DbHandle, LmdbError>
  where
    D: Borrow<DbName>,
  {
    match self.open_dbs.get(name.borrow()) {
      Some(handle) => Ok(*handle),
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
      None => Ok(()),
    }
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

  pub fn stale_reader_check(&self) -> Result<usize, LmdbError> {
    let mut cleared = 0;
    result_fn!(mdb_reader_check(self.handle, &mut cleared))?;

    // NOTE: cleared count must B-positive!
    Ok(cleared.max(0) as usize)
  }

  pub fn reader_list_formatted(&self) -> Result<String, LmdbError> {
    let mut bytes: Vec<u8> = Vec::new();

    {
      let vec_ptr: *mut Vec<u8> = &mut bytes;

      result_fn!(mdb_reader_list(
        self.handle,
        Some(Self::reader_callback),
        vec_ptr.cast()
      ))?;
    }

    let result = String::from_utf8(bytes).map_err(|_| ErrorKind::NonUtf8Str)?;
    Ok(result)
  }

  pub fn reader_list(&self) -> Result<Vec<ReaderInfo>, LmdbError> {
    let output = self.reader_list_formatted()?;
    parse_reader_list(&output).map_err(|_| ErrorKind::ReaderListParseError.into())
  }

  #[inline]
  pub fn close(self) {
    drop(self)
  }

  extern "C" fn reader_callback(msg: *const c_char, ctx: *mut c_void) -> c_int {
    const INVALID_PARAM: c_int = 20;

    if msg.is_null() {
      return INVALID_PARAM;
    }

    match unsafe { ctx.cast::<Vec<u8>>().as_mut() } {
      Some(vec) => {
        let msg = unsafe { CStr::from_ptr(msg) };
        vec.extend_from_slice(msg.to_bytes());
        0
      }
      None => INVALID_PARAM,
    }
  }
}

impl<R, W> AsRawPtr for Env<R, W>
where
  R: ReadMarker,
  W: WriteMarker,
{
  type Return = MDB_env;

  #[inline]
  fn as_raw_ptr(&self) -> *mut Self::Return {
    self.handle
  }
}

unsafe impl<R, W> Send for Env<R, W>
where
  R: ReadMarker + Send,
  W: WriteMarker + Send,
{
}

unsafe impl<R, W> Sync for Env<R, W>
where
  R: ReadMarker + Sync,
  W: WriteMarker + Sync,
{
}

impl<R, W> Drop for Env<R, W>
where
  R: ReadMarker,
  W: WriteMarker,
{
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
