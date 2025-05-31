use std::borrow::Cow;

pub fn escape_nut_str(text: &str) -> Cow<'_, str> {
  let mut escaped = String::new();
  let mut slice_start: usize = 0;

  for (idx, char_byte) in text.as_bytes().iter().enumerate() {
    if *char_byte == b'"' || *char_byte == b'\\' {
      if slice_start < idx {
        escaped.push_str(&text[slice_start..idx]);
      }

      escaped.push('\\');
      escaped.push(char::from(*char_byte));
      slice_start = idx + 1;
    } else {
      continue;
    }
  }

  if escaped.is_empty() {
    Cow::Borrowed(text)
  } else {
    if slice_start < text.len() {
      escaped.push_str(&text[slice_start..]);
    }

    Cow::Owned(escaped)
  }
}

#[cfg(test)]
mod test {
  use super::escape_nut_str;

  macro_rules! assert_escape {
    ($input:expr, owned = $target:expr) => {
      match escape_nut_str($input) {
        std::borrow::Cow::Owned(v) => assert_eq!(v, $target),
        std::borrow::Cow::Borrowed(_) => assert!(false, "it should've allocate new string"),
      }
    };

    ($input:expr, borrowed = $target:expr) => {
      match escape_nut_str($input) {
        std::borrow::Cow::Borrowed(v) => assert_eq!(v, $target),
        std::borrow::Cow::Owned(_) => assert!(false, "it should not allocate new string"),
      }
    };
  }

  #[test]
  fn empty_text() {
    assert_escape!("", borrowed = "");
  }

  #[test]
  fn regular_texts() {
    assert_escape!("abcds", borrowed = "abcds");
    assert_escape!(" test abc dfg ", borrowed = " test abc dfg ");
    assert_escape!("1", borrowed = "1");
    assert_escape!("£$%^&*()[]!", borrowed = "£$%^&*()[]!");
    assert_escape!("/test/example/", borrowed = "/test/example/");
  }

  #[test]
  fn escaped_texts() {
    assert_escape!("\"test\"", owned = "\\\"test\\\"");
    assert_escape!(" test \\abc\\ dfg ", owned = " test \\\\abc\\\\ dfg ");
    assert_escape!("\\\"1\\\"", owned = "\\\\\\\"1\\\\\\\"");
  }
}
