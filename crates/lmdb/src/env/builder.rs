use super::{
  Env, ReadMarker, WriteMarker,
  env_reader::MdbReader,
  env_tls_reader::MdbTlsReader,
  env_writer::{MdbEmptyWriter, MdbWriter},
};
use crate::{error::LmdbError, flag::EnvFlag, internal::ffi::mdb_mode_t};
use core::{
  marker::PhantomData,
  num::{NonZeroU32, NonZeroUsize},
};
use std::path::Path;

pub struct Builder<R, W>
where
  R: ReadMarker,
  W: WriteMarker,
{
  flags: EnvFlag,
  map_size: Option<NonZeroUsize>,
  max_dbs: Option<NonZeroU32>,
  max_readers: Option<NonZeroU32>,
  page_size: Option<NonZeroUsize>,
  permissions: mdb_mode_t,
  phantom_r: PhantomData<R>,
  phantom_w: PhantomData<W>,
}

impl<R, W> Builder<R, W>
where
  R: ReadMarker,
  W: WriteMarker,
{
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

  pub const fn set_flags(mut self, mut value: EnvFlag) -> Self {
    if self.flags.has(EnvFlag::NO_TLS) {
      value.set_assign(EnvFlag::NO_TLS);
    } else {
      value.unset_assign(EnvFlag::NO_TLS);
    }

    if self.flags.has(EnvFlag::READONLY) {
      value.set_assign(EnvFlag::READONLY);
    } else {
      value.unset_assign(EnvFlag::READONLY);
    }

    self.flags = value;
    self
  }

  #[inline]
  pub const fn set_permissions(mut self, value: u32) -> Self {
    self.permissions = value as mdb_mode_t;
    self
  }

  /// Disables MDB_NOTLS and ties locktable slots to thread local storage.
  #[inline]
  pub const fn enable_thread_local_reader(self) -> Builder<MdbTlsReader, W> {
    Builder {
      flags: self.flags.unset(EnvFlag::NO_TLS),
      map_size: self.map_size,
      max_dbs: self.max_dbs,
      max_readers: self.max_readers,
      page_size: self.page_size,
      permissions: self.permissions,
      phantom_r: PhantomData,
      phantom_w: PhantomData,
    }
  }

  /// Opens environment with write support
  #[inline]
  pub const fn enable_write(self) -> Builder<R, MdbWriter> {
    Builder {
      flags: self.flags.unset(EnvFlag::READONLY),
      map_size: self.map_size,
      max_dbs: self.max_dbs,
      max_readers: self.max_readers,
      page_size: self.page_size,
      permissions: self.permissions,
      phantom_r: PhantomData,
      phantom_w: PhantomData,
    }
  }

  pub fn open<P>(self, path: P) -> Result<Env<R, W>, LmdbError>
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

    env.open(path.as_ref(), self.flags, self.permissions)?;

    Ok(env)
  }
}

// NOTE: initializers builder with proper component type and default flags.
macro_rules! impl_initializer_fn {
  ($reader:ty, $writer:ty, $default_flag:expr) => {
    impl Builder<$reader, $writer> {
      #[inline]
      pub const fn new() -> Self {
        Self {
          flags: $default_flag,
          map_size: None,
          max_dbs: None,
          max_readers: None,
          page_size: None,
          permissions: 0o644,
          phantom_r: PhantomData,
          phantom_w: PhantomData,
        }
      }
    }
  };
}

impl_initializer_fn!(MdbTlsReader, MdbWriter, EnvFlag::new());
impl_initializer_fn!(
  MdbTlsReader,
  MdbEmptyWriter,
  EnvFlag::new().set(EnvFlag::READONLY)
);

impl_initializer_fn!(MdbReader, MdbWriter, EnvFlag::new().set(EnvFlag::NO_TLS));
impl_initializer_fn!(
  MdbReader,
  MdbEmptyWriter,
  EnvFlag::new().set(EnvFlag::NO_TLS).set(EnvFlag::READONLY)
);
