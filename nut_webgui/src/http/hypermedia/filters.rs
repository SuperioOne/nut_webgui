use std::borrow::Cow;

pub fn normalize_id(input: &str) -> Cow<'_, str> {
  let first = input.as_bytes().get(0);

  let input = if first.is_some_and(|v| v.is_ascii_digit()) {
    let mut prefixed = String::new();
    prefixed.push('_');
    prefixed.push_str(input);

    Cow::Owned(prefixed)
  } else {
    Cow::Borrowed(input)
  };

  let mut iter = input.as_bytes().iter();
  while let Some(ch) = iter.next() {
    if !ch.is_ascii_alphanumeric() && *ch != b'_' && *ch != b'-' && *ch != b'.' {
      let escaped = input.replace(
        |input: char| {
          !input.is_ascii_alphanumeric() && input != '.' && input != '_' && input != '-'
        },
        "_",
      );

      return Cow::Owned(escaped);
    }
  }

  input
}
