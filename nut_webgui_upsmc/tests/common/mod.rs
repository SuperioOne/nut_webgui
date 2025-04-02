use core::panic;
use std::{fmt::Write, fs::File, io::Read, path::Path};

pub struct DeviceDumpFile {
  file: File,
  ups_name: String,
}

impl DeviceDumpFile {
  pub fn new(ups_name: &str, path: &Path) -> Result<Self, std::io::Error> {
    let file = File::open(path)?;

    Ok(Self {
      file,
      ups_name: ups_name.to_owned(),
    })
  }

  /// Reads all dump file contents and converts it to [RFC9271 LIST VAR](https://www.rfc-editor.org/rfc/rfc9271.html#section-4.2.7.7) response
  pub fn into_list_var_response(mut self) -> Result<String, std::io::Error> {
    let mut buf = String::new();
    self.file.read_to_string(&mut buf)?;

    Ok(device_dump_to_rfc9271(&self.ups_name, &buf))
  }
}

fn device_dump_to_rfc9271(ups_name: &str, dump_text: &str) -> String {
  let mut response = format!("BEGIN LIST VAR {}\n", ups_name);

  for line in dump_text.lines() {
    let trimmed = line.trim();

    if !trimmed.starts_with('#') && !trimmed.is_empty() {
      let var_line = {
        match trimmed.split_once('#') {
          Some((data_line, _comment)) => data_line.trim(),
          None => trimmed,
        }
      };

      match var_line.split_once(':') {
        Some((name, value)) => {
          response
            .write_fmt(format_args!(
              "VAR {} {} \"{}\"\n",
              ups_name,
              name.trim(),
              value.trim().replace("\\", "\\\\").replace("\"", "\\\"") // Escapes double quotes and
                                                                       // backslashes
            ))
            .expect("Cannot write into response.");
        }
        _ => panic!("Invalid ddl format."),
      }
    }
  }

  response
    .write_fmt(format_args!("END LIST VAR {}", ups_name))
    .expect("Cannot end response.");

  response
}

#[macro_export]
macro_rules! ups_validation_test {
  (test_name = $test_name:ident , dump_file = $path:expr, ups_name = $ups_name:expr) => {
    #[test]
    fn $test_name() {
      let path = std::path::Path::new($path);
      let ddf = $crate::common::DeviceDumpFile::new($ups_name, &path).unwrap();
      let ups_var_list = ddf.into_list_var_response().unwrap();
      let var_count = (&ups_var_list).lines().count() - 2;

      match nut_webgui_upsmc::parser::parse_var_list(&ups_var_list) {
        Ok(result) => {
          assert_eq!(var_count, result.len());
        }
        Err(err) => {
          assert!(
            false,
            "Parsing failed for {} with error {:?}",
            $ups_name, err
          );
        }
      }
    }
  };
}
