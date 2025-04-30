macro_rules! lookup_table {
  ($name:ident, $size:literal, $(($r_start:literal$(..$r_end:literal)?);)+) => {
    const $name: [bool; $size] = {
      let mut table = [false; $size];
      let mut i: isize = 0;

      let table_ptr = table.as_mut_ptr();

      while i < $size {
        let cell = unsafe { table_ptr.offset(i) };

        let val = match i as u8 {
          $($r_start$(..$r_end)? => true,)+
          _ => false,
        };

        unsafe { cell.write(val); }
        i += 1;
      }

      table
    };
  };
}

lookup_table!(LOOKUP_ASCII_UPS, 128,
  (b'-');
  (b'.');
  (b'_');
  (b'0'..b'9');
  (b'A'..b'Z');
  (b'a'..b'z');
);

lookup_table!(LOOKUP_ASCII_VAR, 128,
  (b'.');
  (b'_');
  (b'0'..b'9');
  (b'a'..b'z');
);

pub trait NutAsciiText {
  fn is_ascii_nut_ups(&self) -> bool;
  fn is_ascii_nut_cmd(&self) -> bool;
  fn is_ascii_nut_var(&self) -> bool;
}

impl NutAsciiText for u8 {
  #[inline]
  fn is_ascii_nut_ups(&self) -> bool {
    match LOOKUP_ASCII_UPS.get((*self) as usize) {
      Some(true) => true,
      _ => false,
    }
  }

  #[inline]
  fn is_ascii_nut_cmd(&self) -> bool {
    match *self {
      b'.' => true,
      b'a'..b'z' => true,
      _ => false,
    }
  }

  #[inline]
  fn is_ascii_nut_var(&self) -> bool {
    match LOOKUP_ASCII_VAR.get((*self) as usize) {
      Some(true) => true,
      _ => false,
    }
  }
}
