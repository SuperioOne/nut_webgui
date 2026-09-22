use super::{
  bitflag::define_bitflag,
  ffi::{
    MDB_APPEND, MDB_APPENDDUP, MDB_CP_COMPACT, MDB_CREATE, MDB_CURRENT, MDB_DUPFIXED, MDB_DUPSORT,
    MDB_ENCRYPT, MDB_FIXEDMAP, MDB_INTEGERDUP, MDB_INTEGERKEY, MDB_MAPASYNC, MDB_MULTIPLE,
    MDB_NODUPDATA, MDB_NOLOCK, MDB_NOMEMINIT, MDB_NOMETASYNC, MDB_NOOVERWRITE, MDB_NORDAHEAD,
    MDB_NOSUBDIR, MDB_NOSYNC, MDB_NOTLS, MDB_PREVSNAPSHOT, MDB_RDONLY, MDB_REMAP_CHUNKS,
    MDB_RESERVE, MDB_REVERSEDUP, MDB_REVERSEKEY, MDB_WRITEMAP,
  },
};

define_bitflag!(
  /// Environment flags
  pub EnvFlag u32,
  values = [
  /// mmap at a fixed address (experimental)
  (FIXED_MAP     = MDB_FIXEDMAP),
  /// encrypted DB - read-only flag, set by #mdb_env_set_encrypt()
  (ENCRYPT       = MDB_ENCRYPT),
   /// no environment directory
  (NO_SUB_DIR    = MDB_NOSUBDIR),
   /// don't fsync after commit
  (NO_SYNC       = MDB_NOSYNC),
   /// read only
  (READONLY      = MDB_RDONLY),
   /// don't fsync metapage after commit
  (NO_META_SYNC  = MDB_NOMETASYNC),
   /// use writable mmap
  (WRITE_MAP     = MDB_WRITEMAP),
  /// use asynchronous msync when #MDB_WRITEMAP is used
  (MAP_ASYNC     = MDB_MAPASYNC),
  /// tie reader locktable slots to #MDB_txn objects instead of to threads
  (NO_TLS        = MDB_NOTLS),
  /// don't do any locking, caller must manage their own locks
  (NO_LOCK       = MDB_NOLOCK),
  /// don't do readahead (no effect on Windows)
  (NO_READ_AHEAD = MDB_NORDAHEAD),
  /// don't initialize malloc'd memory before writing to datafile
  (NO_MEM_INIT   = MDB_NOMEMINIT),
  /// use the previous snapshot rather than the latest one
  (PREV_SNAPSHOT = MDB_PREVSNAPSHOT),
  /// don't use a single mmap, remap individual chunks (needs MDB_RPAGE_CACHE)
  (REMAP_CHUNKS  = MDB_REMAP_CHUNKS)
  ]
);

define_bitflag!(
  /// Database flags
  pub DbFlag u32,
  values = [
  /// use reverse string keys
  (REVERSE_KEY = MDB_REVERSEKEY),
  /// use sorted duplicates
  (DUP_SORT    = MDB_DUPSORT),
  /// numeric keys in native byte order, either unsigned int or #mdb_size_t.
  /// The keys must all be of the same size.
  (INTEGER_KEY = MDB_INTEGERKEY),
  /// with #MDB_DUPSORT, sorted dup items have fixed size
  (DUP_FIXED   = MDB_DUPFIXED),
  /// with #MDB_DUPSORT, dups are #MDB_INTEGERKEY-style integers
  (INTEGER_DUP = MDB_INTEGERDUP),
  /// with #MDB_DUPSORT, use reverse string dups
  (REVERSE_DUP = MDB_REVERSEDUP),
  /// create DB if not already existing
  (CREATE      = MDB_CREATE)
  ]
);

define_bitflag!(
  /// Writeflags
  pub WriteFlag u32,
  values = [
  /// For put: Don't write if the key already exists.
  (NO_OVERWRITE = MDB_NOOVERWRITE),
  /// Only for #MDB_DUPSORT
  /// For put: don't write if the key and data pair already exist.
  /// For mdb_cursor_del: remove all duplicate data items.
  (NO_DUPDATA   = MDB_NODUPDATA),
  /// For mdb_cursor_put: overwrite the current key/data pair
  (CURRENT      = MDB_CURRENT),
  /// For put: Just reserve space for data, don't copy it. Return a pointer to
  /// the reserved space.
  (RESERVE      = MDB_RESERVE),
  /// Data is being appended, don't split full pages.
  (APPEND       = MDB_APPEND),
  /// Duplicate data is being appended, don't split full pages.
  (APPEND_DUP   = MDB_APPENDDUP),
  /// Store multiple data items in one call. Only for #MDB_DUPFIXED.
  (MULTIPLE     = MDB_MULTIPLE)
  ]
);

define_bitflag!(
  pub CopyFlag u32,
  values = [
  /// Compacting copy: Omit free space from copy, and renumber all pages
  /// sequentially.
  (CP_COMPACT = MDB_CP_COMPACT)
  ]
);
