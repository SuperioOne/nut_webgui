use crate::ffi::MDB_val;
use core::{borrow::Borrow, marker::PhantomData};
use std::ptr::null_mut;

pub struct ValueRef<'a> {
  val: MDB_val,
  _phantom: PhantomData<&'a *const MDB_val>,
}

pub trait IntoValueRef: Sized {
  fn as_value_ref(&self) -> ValueRef<'_>;
}

impl<'a> ValueRef<'a> {
  #[inline]
  pub const fn as_mdb_val(&self) -> &MDB_val {
    &self.val
  }

  #[inline]
  pub(crate) const fn from_mdb_val(val: MDB_val) -> Self {
    Self {
      val,
      _phantom: PhantomData,
    }
  }

  #[inline]
  pub const fn len(&self) -> usize {
    self.val.mv_size
  }

  #[inline]
  pub const fn is_empty(&self) -> bool {
    self.val.mv_size == 0
  }

  pub fn as_bytes(&self) -> &[u8] {
    if self.val.mv_data == null_mut() {
      &[]
    } else {
      unsafe {
        core::slice::from_raw_parts(self.val.mv_data.cast_const().cast::<u8>(), self.val.mv_size)
      }
    }
  }
}

impl<'a> AsRef<MDB_val> for ValueRef<'a> {
  #[inline]
  fn as_ref(&self) -> &MDB_val {
    self.as_mdb_val()
  }
}

impl<'a> Borrow<MDB_val> for ValueRef<'a> {
  #[inline]
  fn borrow(&self) -> &MDB_val {
    self.as_mdb_val()
  }
}

macro_rules! impl_into_value_ref {
  ($type:ty) => {
    impl IntoValueRef for $type {
      #[inline]
      fn as_value_ref(&self) -> ValueRef<'_> {
        ValueRef {
          val: MDB_val {
            mv_size: size_of::<$type>(),
            mv_data: core::ptr::from_ref(self).cast_mut().cast(),
          },
          _phantom: PhantomData,
        }
      }
    }
  };
}

impl_into_value_ref!(bool);
impl_into_value_ref!(f32);
impl_into_value_ref!(f64);
impl_into_value_ref!(i128);
impl_into_value_ref!(i16);
impl_into_value_ref!(i32);
impl_into_value_ref!(i64);
impl_into_value_ref!(i8);
impl_into_value_ref!(isize);
impl_into_value_ref!(u128);
impl_into_value_ref!(u16);
impl_into_value_ref!(u32);
impl_into_value_ref!(u64);
impl_into_value_ref!(u8);
impl_into_value_ref!(usize);

impl<T> IntoValueRef for Vec<T>
where
  T: Sized,
{
  fn as_value_ref(&self) -> ValueRef<'_> {
    ValueRef {
      val: MDB_val {
        mv_size: self.len() * size_of::<T>(),
        mv_data: self.as_ptr().cast_mut().cast(),
      },
      _phantom: PhantomData,
    }
  }
}

impl<T> IntoValueRef for &[T]
where
  T: Sized,
{
  fn as_value_ref(&self) -> ValueRef<'_> {
    ValueRef {
      val: MDB_val {
        mv_size: self.len() * size_of::<T>(),
        mv_data: self.as_ptr().cast_mut().cast(),
      },
      _phantom: PhantomData,
    }
  }
}

impl<T, const S: usize> IntoValueRef for [T; S]
where
  T: Sized,
{
  fn as_value_ref(&self) -> ValueRef<'_> {
    ValueRef {
      val: MDB_val {
        mv_size: S * size_of::<T>(),
        mv_data: self.as_ptr().cast_mut().cast(),
      },
      _phantom: PhantomData,
    }
  }
}

impl IntoValueRef for &str {
  fn as_value_ref(&self) -> ValueRef<'_> {
    let bytes = self.as_bytes();
    ValueRef {
      val: MDB_val {
        mv_size: bytes.len(),
        mv_data: bytes.as_ptr().cast_mut().cast(),
      },
      _phantom: PhantomData,
    }
  }
}

impl IntoValueRef for String {
  fn as_value_ref(&self) -> ValueRef<'_> {
    let bytes = self.as_bytes();
    ValueRef {
      val: MDB_val {
        mv_size: bytes.len(),
        mv_data: bytes.as_ptr().cast_mut().cast(),
      },
      _phantom: PhantomData,
    }
  }
}
