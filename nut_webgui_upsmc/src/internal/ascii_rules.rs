// Subset of ASCII to match RFC9271
const LOOKUP_ASCII_UPS: [bool; 128] = {
  let mut table = [false; 128];
  let mut i: isize = 0;
  let table_ptr = table.as_mut_ptr();

  while i < 128 {
    let cell = unsafe { table_ptr.offset(i) };
    let val = match i as u8 {
      b'-' | b'.' | b'0'..=b'9' | b'A'..=b'Z' | b'_' | b'a'..=b'z' => true,
      _ => false,
    };

    unsafe {
      cell.write(val);
    }

    i += 1;
  }

  table
};

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
    match LOOKUP_ASCII_UPS.get((*self) as usize) {
      Some(true) => true,
      _ => false,
    }
  }

  #[inline]
  fn is_ascii_nut_var(&self) -> bool {
    match LOOKUP_ASCII_UPS.get((*self) as usize) {
      Some(true) => true,
      _ => false,
    }
  }
}
