use std::borrow::Cow;

const ESCAPE_BYTE: u8 = b'\\';
const ESCAPE_CHAR: char = '\\';
const QUOTE_BYTE: u8 = b'"';

pub struct AsciiWords<'a> {
  inner: Vec<Cow<'a, str>>,
}

// NOTE: Insert `Must Not` meme to limit the urge for creating SIMD version.

/// Splits US-ASCII text into words based on notation defined in RFC 9271 with minimal allocations.
impl<'a> AsciiWords<'a> {
  /// Internal only
  /// If necessary, allocates new String without escape characters.
  fn into_escaped(word: &'a str) -> Cow<'a, str> {
    if let Some(first_idx) = word.find(ESCAPE_CHAR) {
      let mut escaped_word = String::from(&word[..first_idx]);
      let mut slice_start: usize = 0;
      let remaining = &word[first_idx + 1..];

      for (idx, character) in remaining.as_bytes().iter().enumerate() {
        if *character == ESCAPE_BYTE {
          if slice_start < idx {
            escaped_word.push_str(&remaining[slice_start..idx]);
            slice_start = idx + 1;
          }
        }
      }

      if slice_start < remaining.len() {
        escaped_word.push_str(&remaining[slice_start..]);
      }

      Cow::Owned(escaped_word)
    } else {
      Cow::Borrowed(word)
    }
  }

  pub fn split(input: &'a str) -> Self {
    let mut words: Vec<Cow<'a, str>> = Vec::new();
    let mut slice_start: Option<usize> = None;
    let mut escape: bool = false;
    let mut slice_char: Option<u8> = None;

    for (idx, char_byte) in input.as_bytes().iter().enumerate() {
      if escape {
        escape = false;
      } else if *char_byte == QUOTE_BYTE && slice_char != Some(QUOTE_BYTE) {
        slice_char = Some(QUOTE_BYTE);
        slice_start = Some(idx + 1);
      } else if slice_char.is_some_and(|sc| sc == *char_byte)
        || (slice_char.is_none() && char_byte.is_ascii_whitespace())
      {
        if let Some(start) = slice_start {
          words.push(Self::into_escaped(&input[start..idx]));
        }

        slice_start = None;
        slice_char = None;
      } else if *char_byte == ESCAPE_BYTE {
        escape = true;

        if slice_start.is_none() {
          slice_start = Some(idx);
        }
      } else if slice_start.is_none() {
        slice_start = Some(idx);
      }
    }

    if let Some(start) = slice_start {
      words.push(Self::into_escaped(&input[start..]));
    }

    Self { inner: words }
  }

  /// Returns splitted words as slice.
  #[inline]
  pub fn as_slice(&self) -> &[Cow<'a, str>] {
    &self.inner
  }
}

#[cfg(test)]
mod test {
  use crate::internal::word_split::AsciiWords;

  macro_rules! split_test {
    (test_name = $test_name:ident, input = $input:expr, expected = $expected:expr) => {
      #[test]
      fn $test_name() {
        let input: &str = $input;
        let expected: &[&str] = &$expected;
        let words = AsciiWords::split(input);

        assert_eq!(words.as_slice().len(), expected.len());

        for (word, expected) in AsciiWords::split(input).as_slice().iter().zip(expected) {
          assert_eq!(expected, word);
        }
      }
    };
  }

  split_test!(
    test_name = split_str_multi_word,
    input = "example split \\w\\\"ith complex\\ words",
    expected = ["example", "split", "w\"ith", "complex words"]
  );

  split_test!(
    test_name = split_str_single_word,
    input = "single_word",
    expected = ["single_word"]
  );

  split_test!(
    test_name = split_str_variable_spaces,
    input = "    example     split with           spaces          ",
    expected = ["example", "split", "with", "spaces"]
  );

  split_test!(
    test_name = split_str_quoted_words,
    input = "\"left example words\" \"centered test words\" \"right word with trailing space \"",
    expected = [
      "left example words",
      "centered test words",
      "right word with trailing space "
    ]
  );

  split_test!(test_name = split_str_empty, input = "", expected = []);

  split_test!(
    test_name = split_str_whitespace,
    input = "         ",
    expected = []
  );

  split_test!(
    test_name = split_str_empty_quoted,
    input = "\"\"",
    expected = [""]
  );

  split_test!(
    test_name = split_str_escaped_chars,
    input = "\"double quoted \\\"text\\\" is here.\" escaped\\ space\\ and\\ reverse\\ slash\\ (\\\\) \\\\\\\\\\\\\\\\",
    expected = [
      "double quoted \"text\" is here.",
      "escaped space and reverse slash (\\)",
      "\\\\\\\\"
    ]
  );

  split_test!(
    test_name = split_str_ascii_whitespaces,
    input = "\tnew\tline\ttest\n",
    expected = ["new", "line", "test"]
  );
}
