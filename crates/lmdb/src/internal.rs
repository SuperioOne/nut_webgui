pub mod bitflag;
pub mod ffi;

pub trait AsRawPtr {
  type Return;
  fn as_raw_ptr(&self) -> *mut Self::Return;
}
