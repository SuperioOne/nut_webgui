/// Checks path for both URI path and Axum 0.7 route syntax.
/// See also: [RFC 3986](https://www.rfc-editor.org/rfc/rfc3986#section-3.3)
fn is_valid_path(path: &str) -> bool {
  let mut prev_chr: char = '\0';

  for chr in path.chars() {
    if chr == '/' && prev_chr == '/' {
      return false;
    }

    match chr {
      // sub-delims
       '!'
      | '$'
      | '&'
      | '('
      | ')'
      | '+'
      | ','
      | ';'
      | '='
      // unreserved
      | '-'
      | '.'
      | '_'
      | '~'
      // Alpha numeric
      | 'a'..='z'
      | 'A'..='Z'
      | '0'..='9'
      // special terminals
      | '@'
      | '/'
      => {
        prev_chr = chr;
        continue;
      }
      // Anything else
      _ => return false,
    }
  }

  true
}

#[derive(Debug)]
pub struct InvalidBasePathError;

impl std::fmt::Display for InvalidBasePathError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("incorrect base path format")
  }
}

pub fn get_base_path(input: Option<String>) -> Result<String, InvalidBasePathError> {
  match input {
    Some(path) => {
      let normalized = path
        .trim()
        .trim_end_matches(|char: char| char.is_whitespace() || char == '/');

      if is_valid_path(&normalized) {
        if normalized.is_empty() {
          Ok(String::new())
        } else if normalized.starts_with('/') {
          Ok(normalized.into())
        } else {
          Ok(format!("/{}", normalized))
        }
      } else {
        Err(InvalidBasePathError)
      }
    }
    None => Ok(String::new()),
  }
}
