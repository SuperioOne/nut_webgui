use std::borrow::Cow;

#[cfg(test)]
mod test;

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
