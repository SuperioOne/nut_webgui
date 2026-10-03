use crate::DbHandle;
use crate::error::LmdbError;
use crate::internal::AsRawPtr;
use crate::internal::ffi::{
  MDB_cursor, MDB_cursor_op_MDB_FIRST, MDB_cursor_op_MDB_FIRST_DUP, MDB_cursor_op_MDB_GET_BOTH,
  MDB_cursor_op_MDB_GET_BOTH_RANGE, MDB_cursor_op_MDB_GET_CURRENT, MDB_cursor_op_MDB_GET_MULTIPLE,
  MDB_cursor_op_MDB_LAST, MDB_cursor_op_MDB_LAST_DUP, MDB_cursor_op_MDB_NEXT,
  MDB_cursor_op_MDB_NEXT_DUP, MDB_cursor_op_MDB_NEXT_MULTIPLE, MDB_cursor_op_MDB_NEXT_NODUP,
  MDB_cursor_op_MDB_PREV, MDB_cursor_op_MDB_PREV_DUP, MDB_cursor_op_MDB_PREV_MULTIPLE,
  MDB_cursor_op_MDB_PREV_NODUP, MDB_cursor_op_MDB_SET, MDB_cursor_op_MDB_SET_KEY,
  MDB_cursor_op_MDB_SET_RANGE, MDB_txn, mdb_cursor_close, mdb_cursor_dbi, mdb_cursor_is_db,
  mdb_cursor_open, result_fn,
};
use crate::transaction::Transaction;
use core::marker::PhantomData;
use std::ptr::null_mut;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum CursorOp {
  /// Position at first key/data item
  First = MDB_cursor_op_MDB_FIRST,
  /// Position at first data item of current key. Only for #MDB_DUPSORT
  FirstDup = MDB_cursor_op_MDB_FIRST_DUP,
  /// Position at key/data pair. Only for #MDB_DUPSORT
  GetBoth = MDB_cursor_op_MDB_GET_BOTH,
  /// position at key, nearest data. Only for #MDB_DUPSORT
  GetBothRange = MDB_cursor_op_MDB_GET_BOTH_RANGE,
  /// Return key/data at current cursor position
  GetCurrent = MDB_cursor_op_MDB_GET_CURRENT,
  /// Return up to a page of duplicate data items from current cursor position.
  /// Move cursor to prepare for #MDB_NEXT_MULTIPLE. Only for #MDB_DUPFIXED
  GetMultiple = MDB_cursor_op_MDB_GET_MULTIPLE,
  /// Position at last key/data item
  Last = MDB_cursor_op_MDB_LAST,
  /// Position at last data item of current key. Only for #MDB_DUPSORT
  LastDup = MDB_cursor_op_MDB_LAST_DUP,
  /// Position at next data item
  Next = MDB_cursor_op_MDB_NEXT,
  /// Position at next data item of current key. Only for #MDB_DUPSORT
  NextDup = MDB_cursor_op_MDB_NEXT_DUP,
  /// Return up to a page of duplicate data items from next cursor position.
  /// Move cursor to prepare for #MDB_NEXT_MULTIPLE. Only for #MDB_DUPFIXED
  NextMultiple = MDB_cursor_op_MDB_NEXT_MULTIPLE,
  /// Position at first data item of next key
  NextNodup = MDB_cursor_op_MDB_NEXT_NODUP,
  /// Position at previous data item
  Prev = MDB_cursor_op_MDB_PREV,
  /// Position at previous data item of current key. Only for #MDB_DUPSORT
  PrevDup = MDB_cursor_op_MDB_PREV_DUP,
  /// Position at last data item of previous key
  PrevNodup = MDB_cursor_op_MDB_PREV_NODUP,
  /// Position at specified key
  Set = MDB_cursor_op_MDB_SET,
  /// Position at specified key, return key + data
  SetKey = MDB_cursor_op_MDB_SET_KEY,
  /// Position at first key greater than or equal to specified key.
  SetRange = MDB_cursor_op_MDB_SET_RANGE,
  /// Position at previous page and return up to a page of duplicate data items.
  /// Only for #MDB_DUPFIXED */
  PrevMultiple = MDB_cursor_op_MDB_PREV_MULTIPLE,
}

pub struct Cursor<'a> {
  handle: *mut MDB_cursor,
  _phantom: PhantomData<&'a MDB_cursor>,
}

impl<'a> Cursor<'a> {
  pub fn new<T>(txn: T, db_handle: DbHandle) -> Result<Self, LmdbError>
  where
    T: Transaction + AsRawPtr<Return = MDB_txn>,
  {
    let mut cursor = Self {
      handle: null_mut(),
      _phantom: PhantomData,
    };

    result_fn!(mdb_cursor_open(
      txn.as_raw_ptr(),
      db_handle,
      &mut cursor.handle
    ))?;

    Ok(cursor)
  }

  pub fn is_db(&self) -> bool {
    unsafe { mdb_cursor_is_db(self.handle) == 0 }
  }

  pub fn db_handle(&self) -> DbHandle {
    unsafe { mdb_cursor_dbi(self.handle) }
  }

  #[inline]
  pub fn close(self) {
    drop(self)
  }
}

impl<'a> AsRawPtr for Cursor<'a> {
  type Return = MDB_cursor;

  #[inline]
  fn as_raw_ptr(&self) -> *mut Self::Return {
    self.handle
  }
}

impl Drop for Cursor<'_> {
  fn drop(&mut self) {
    if !self.handle.is_null() {
      unsafe {
        mdb_cursor_close(self.handle);
      }
    }
  }
}
