use super::ffi::{
  MDB_ADDR_BUSY, MDB_BAD_CHECKSUM, MDB_BAD_DBI, MDB_BAD_RSLOT, MDB_BAD_TXN, MDB_BAD_VALSIZE,
  MDB_CANT_ROLLBACK, MDB_CORRUPTED, MDB_CRYPTO_FAIL, MDB_CURSOR_FULL, MDB_DBIS_BUSY, MDB_DBS_FULL,
  MDB_ENV_BUSY, MDB_ENV_ENCRYPTION, MDB_INCOMPATIBLE, MDB_INVALID, MDB_IS_READONLY, MDB_KEYEXIST,
  MDB_MAP_FULL, MDB_MAP_RESIZED, MDB_NOTFOUND, MDB_PAGE_FULL, MDB_PAGE_NOTFOUND, MDB_PANIC,
  MDB_PROBLEM, MDB_READERS_FULL, MDB_SHORT_WRITE, MDB_TLS_FULL, MDB_TXN_FULL, MDB_TXN_PENDING,
  MDB_VERSION_MISMATCH, mdb_strerror,
};
use core::ffi::CStr;

macro_rules! impl_lmdb_errors {
    (
        catch_all = $catch_all:tt,
        repr_type = $type:ty,
        $(
            $(#[$doc:meta])*
            ($variant_name:tt = $value:tt)
        ),+
    ) => {
        #[repr($type)]
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub enum LmdbError {
            $(
                $(#[$doc])*
                $variant_name = $value
            ),+
        }

        impl LmdbError {
            #[inline]
            pub const fn as_value(&self) -> $type {
                *self as $type
            }

            pub const fn from_raw_code(value:$type) -> LmdbError {
                match value {
                    $(
                        $value => LmdbError::$variant_name,
                    )+
                    _ => LmdbError::$catch_all
                }
            }
        }
    };
}

impl_lmdb_errors!(catch_all = Unknown, repr_type = i32,
  /// Unspecified LMDB library error
  (Unknown         = 1),
  /// key/data pair already exists
  (KeyExist        = MDB_KEYEXIST),
  /// key/data pair not found (EOF)
  (NotFound        = MDB_NOTFOUND),
  /// Requested page not found - this usually indicates corruption
  (PageNotFound    = MDB_PAGE_NOTFOUND),
  /// Located page was wrong type
  (Corrupted       = MDB_CORRUPTED),
  /// Update of meta page failed or environment had fatal error
  (Panic           = MDB_PANIC),
  /// Environment version mismatch
  (VersionMismatch = MDB_VERSION_MISMATCH),
  /// File is not a valid LMDB file
  (Invalid         = MDB_INVALID),
  /// Environment mapsize reached
  (MapFull         = MDB_MAP_FULL),
  /// Environment maxdbs reached
  (DbsFull         = MDB_DBS_FULL),
  /// Environment maxreaders reached
  (ReadersFull     = MDB_READERS_FULL),
  /// Too many TLS keys in use - Windows only
  (TlsFull         = MDB_TLS_FULL),
  /// Txn has too many dirty pages
  (TxnFull         = MDB_TXN_FULL),
  /// Cursor stack too deep - internal error
  (CursorFull      = MDB_CURSOR_FULL),
  /// Page has not enough space - internal error
  (PageFull        = MDB_PAGE_FULL),
  /// Database contents grew beyond environment mapsize
  (MapResized      = MDB_MAP_RESIZED),
  /// Operation and DB incompatible, or DB type changed. This can mean:
  /// - The operation expects an #MDB_DUPSORT / #MDB_DUPFIXED database.
  /// - Opening a named DB when the unnamed DB has #MDB_DUPSORT / #MDB_INTEGERKEY.
  /// - Accessing a data record as a database, or vice versa.
  /// - The database was dropped and recreated with different flags.
  (Incompatible    = MDB_INCOMPATIBLE),
  /// Invalid reuse of reader locktable slot
  (BadRslot        = MDB_BAD_RSLOT),
  /// Transaction must abort, has a child, or is invalid
  (BadTxn          = MDB_BAD_TXN),
  /// Unsupported size of key/DB name/data, or wrong DUPFIXED size
  (BadValsize      = MDB_BAD_VALSIZE),
  /// The specified DBI was changed unexpectedly
  (BadDbi          = MDB_BAD_DBI),
  /// Unexpected problem - txn should abort
  (Problem         = MDB_PROBLEM),
  /// Page checksum incorrect
  (BadChecksum     = MDB_BAD_CHECKSUM),
  /// Encryption/decryption failed
  (CryptoFail      = MDB_CRYPTO_FAIL),
  /// Environment encryption mismatch
  (EnvEncryption   = MDB_ENV_ENCRYPTION),
  /// Transaction was already prepared
  (TxnPending      = MDB_TXN_PENDING),
  /// Environment can't rollback the last transaction
  (CantRollback    = MDB_CANT_ROLLBACK),
  /// Can't drop main DBI while other DBIs are open
  (DbisBusy        = MDB_DBIS_BUSY),
  /// Write was incomplete
  (ShortWrite      = MDB_SHORT_WRITE),
  /// Env is busy, can't use previous snapshot
  (EnvBusy         = MDB_ENV_BUSY),
  /// Env or txn is read-only, can't write
  (IsReadonly      = MDB_IS_READONLY),
  /// Requested map address is unavailable
  (AddrBusy        = MDB_ADDR_BUSY)
);

#[derive(Debug)]
pub enum ErrorKind {
  LmdbError(LmdbError),
  IOError(std::io::Error),
  PathError,
}

#[derive(Debug)]
pub struct Error {
  kind: ErrorKind,
}

impl Error {
  pub fn kind(&self) -> &ErrorKind {
    &self.kind
  }

  pub fn result_from_code(code: i32) -> Result<(), Error> {
    if code == 0 {
      Ok(())
    } else if code > 0 {
      Err(ErrorKind::IOError(std::io::Error::from_raw_os_error(code)).into())
    } else {
      Err(ErrorKind::LmdbError(LmdbError::from_raw_code(code)).into())
    }
  }
}

impl From<ErrorKind> for Error {
  #[inline]
  fn from(kind: ErrorKind) -> Self {
    Self { kind }
  }
}

impl From<std::io::Error> for ErrorKind {
  #[inline]
  fn from(error: std::io::Error) -> Self {
    ErrorKind::IOError(error)
  }
}

impl From<LmdbError> for ErrorKind {
  #[inline]
  fn from(error: LmdbError) -> Self {
    ErrorKind::LmdbError(error)
  }
}

impl core::fmt::Display for Error {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self.kind() {
      ErrorKind::LmdbError(lmdb_error) => lmdb_error.fmt(f),
      ErrorKind::IOError(error) => error.fmt(f),
      ErrorKind::PathError => f.write_str("invalid database path"),
    }
  }
}

impl core::fmt::Display for LmdbError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      LmdbError::Unknown => f.write_str("unspecified error code from lmdb"),
      _ => {
        let val = self.as_value();
        let msg_ptr = unsafe { mdb_strerror(val) };
        match unsafe { CStr::from_ptr(msg_ptr) }.to_str() {
          Ok(msg) => f.write_str(msg),
          Err(_) => f.write_str("non-utf8 error message."),
        }
      }
    }
  }
}

impl core::error::Error for Error {}
impl core::error::Error for LmdbError {}
