#![allow(nonstandard_style)]
#![allow(unused)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

macro_rules! result_fn {
  ($fn:expr) => {
    $crate::error::LmdbError::result_from_code(unsafe { $fn })
  };
}

use std::ptr::null_mut;

pub(super) use result_fn;

use crate::EnvInfo;

impl Default for MDB_stat {
  fn default() -> Self {
    Self {
      ms_psize: 0,
      ms_depth: 0,
      ms_branch_pages: 0,
      ms_leaf_pages: 0,
      ms_overflow_pages: 0,
      ms_entries: 0,
    }
  }
}

impl Default for EnvInfo {
  fn default() -> Self {
    Self {
      me_mapaddr: null_mut(),
      me_mapsize: 0,
      me_last_pgno: 0,
      me_last_txnid: 0,
      me_maxreaders: 0,
      me_numreaders: 0,
    }
  }
}
