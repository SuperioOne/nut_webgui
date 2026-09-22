use super::{
  error::{Error, ErrorKind},
  ffi::{
    MDB_env, MDB_stat, mdb_env_create, mdb_env_incr_dumpfd, mdb_env_open, mdb_env_set_mapsize,
    mdb_env_set_maxdbs, mdb_env_set_maxreaders, mdb_env_set_pagesize, mdb_mode_t, result_fn,
  },
  flag::{CopyFlag, EnvFlag},
};
use crate::lmdb::ffi::{
  MDB_envinfo, mdb_env_close, mdb_env_copy2, mdb_env_copyfd2, mdb_env_incr_dump,
  mdb_env_incr_loadfd, mdb_env_info, mdb_env_rollback, mdb_env_set_flags, mdb_env_stat,
};
use core::ptr::null_mut;
use std::{
  ffi::CString,
  fs::File,
  mem::MaybeUninit,
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

pub type EnvInfo = MDB_envinfo;
pub type EnvStats = MDB_stat;

pub struct Env {
  handle: *mut MDB_env,
}

#[inline]
fn path_to_cstr<P>(path: P) -> Result<CString, Error>
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

  fn init() -> Result<Self, Error> {
    let mut env = Self { handle: null_mut() };
    result_fn!(mdb_env_create(&mut env.handle))?;
    Ok(env)
  }

  fn set_map_size(&mut self, size: NonZeroUsize) -> Result<(), Error> {
    result_fn!(mdb_env_set_mapsize(self.handle, size.get()))?;
    Ok(())
  }

  fn set_page_size(&mut self, size: NonZeroUsize) -> Result<(), Error> {
    result_fn!(mdb_env_set_pagesize(self.handle, size.get() as i32))?;
    Ok(())
  }

  fn set_max_dbs(&mut self, max: NonZeroU32) -> Result<(), Error> {
    result_fn!(mdb_env_set_maxdbs(self.handle, max.get()))?;
    Ok(())
  }

  fn set_max_readers(&mut self, max: NonZeroU32) -> Result<(), Error> {
    result_fn!(mdb_env_set_maxreaders(self.handle, max.get()))?;
    Ok(())
  }

  fn set_flags(&mut self, flags: EnvFlag) -> Result<(), Error> {
    result_fn!(mdb_env_set_flags(self.handle, flags.into_inner(), 1))
  }

  fn open(&mut self, path: &Path, permissions: mdb_mode_t) -> Result<(), Error> {
    let path = path_to_cstr(path)?;
    result_fn!(mdb_env_open(self.handle, path.as_ptr(), 0, permissions))?;
    Ok(())
  }

  pub fn rollback_transaction(&self, txnid: usize) -> Result<(), Error> {
    result_fn!(mdb_env_rollback(self.handle, txnid))
  }

  pub fn info(&self) -> Result<EnvInfo, Error> {
    let mut info: MaybeUninit<EnvInfo> = MaybeUninit::uninit();
    result_fn!(mdb_env_info(self.handle, info.as_mut_ptr()))?;
    Ok(unsafe { info.assume_init() })
  }

  pub fn stats(&self) -> Result<EnvStats, Error> {
    let mut stat: MaybeUninit<EnvStats> = MaybeUninit::uninit();
    result_fn!(mdb_env_stat(self.handle, stat.as_mut_ptr()))?;
    Ok(unsafe { stat.assume_init() })
  }

  pub fn incr_dump_to_path<P>(&self, path: P, txnid: usize) -> Result<(), Error>
  where
    P: AsRef<Path>,
  {
    let path = path_to_cstr(path)?;
    result_fn!(mdb_env_incr_dump(self.handle, path.as_ptr(), txnid))
  }

  pub fn incr_dump_to_file(&self, file: &mut File, txnid: usize) -> Result<(), Error> {
    result_fn!(mdb_env_incr_dumpfd(self.handle, file.as_raw_fd(), txnid))
  }

  pub fn load_dump_from_file(&self, file: &File) -> Result<(), Error> {
    result_fn!(mdb_env_incr_loadfd(self.handle, file.as_raw_fd()))
  }

  pub fn copy_to_dir<P>(&self, path: P, flags: CopyFlag) -> Result<(), Error>
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

  pub fn copy_to_file(&self, file: &mut File, flags: CopyFlag) -> Result<(), Error> {
    result_fn!(mdb_env_copyfd2(
      self.handle,
      file.as_raw_fd(),
      flags.into_inner()
    ))
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
  pub const fn set_max_dbs(mut self, value: NonZeroUsize) -> Self {
    self.map_size = Some(value);
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

  pub fn open<P>(self, path: P) -> Result<Env, Error>
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

impl Drop for Env {
  fn drop(&mut self) {
    if !self.handle.is_null() {
      unsafe { mdb_env_close(self.handle) };
    }
  }
}
