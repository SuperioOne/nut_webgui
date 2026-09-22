#![allow(nonstandard_style)]
#![allow(unused)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

macro_rules! result_fn {
  ($fn:expr) => {
    $crate::lmdb::error::Error::result_from_code(unsafe { $fn })
  };
}

pub(super) use result_fn;
