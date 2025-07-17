use brotli::BrotliCompress;
use brotli::enc::BrotliEncoderParams;
use sha2::Digest;
use std::{
  fs::{File, OpenOptions, canonicalize, read_dir},
  io::{Cursor, Read, Write},
  path::{Path, PathBuf},
  process::Command,
  str::FromStr,
};

fn main() -> Result<(), std::io::Error> {
  // NOTE: Maybe I should ditch nodejs and simply use Tailwind rust crates + RsPack/FarmFe ???
  #[cfg(debug_assertions)]
  let target = {
    Command::new("node")
      .args(&["scripts/build.js", "--outdir=target/static/debug"])
      .status()
      .unwrap();

    PathBuf::from_str("target/static/debug").unwrap()
  };

  #[cfg(not(debug_assertions))]
  let target = {
    Command::new("node")
      .args(&[
        "scripts/build.js",
        "--minify",
        "--outdir=target/static/release",
      ])
      .status()
      .unwrap();

    PathBuf::from_str("target/static/release").unwrap()
  };

  create_asset(&target, "NUTWG_CLIENT_CSS", "style.css")?;
  create_asset(&target, "NUTWG_CLIENT_JS", "index.js")?;
  create_asset(&target, "NUTWG_CLIENT_ICON", "icon.svg")?;
  create_asset(&target, "NUTWG_CLIENT_SPRITE_SHEET", "feather-sprite.svg")?;

  output_watch_list("src", &["css", "js", "json", "rs", "svg"])?;

  Ok(())
}

fn create_asset(src_dir: &Path, env_prefix: &str, file_name: &str) -> Result<(), std::io::Error> {
  let params = BrotliEncoderParams::default();

  let src_dir = {
    if src_dir.is_relative() {
      canonicalize(src_dir)?
    } else {
      src_dir.to_path_buf()
    }
  };

  let file_path = src_dir.join(file_name);

  let mut compressed_path = file_path.clone();
  compressed_path.set_extension("brotli");

  let mut src = File::open(&file_path)?;
  let mut opts = OpenOptions::new();
  opts.create(true).write(true).truncate(true);

  let mut compressed = opts.open(&compressed_path)?;

  let mut content: Vec<u8> = Vec::new();
  src.read_to_end(&mut content)?;

  let sha256 = calc_sha256(&content)?;
  let mut reader = Cursor::new(content);

  BrotliCompress(&mut reader, &mut compressed, &params)?;

  println!(
    "cargo::rustc-env={env_prefix}_PATH={}",
    compressed_path
      .to_str()
      .expect("Not a valid unicode path string")
  );
  println!("cargo::rustc-env={env_prefix}_NAME={file_name}");
  println!("cargo::rustc-env={env_prefix}_SHA256={sha256}",);

  Ok(())
}

fn calc_sha256(bytes: &[u8]) -> Result<String, std::io::Error> {
  let mut sha256 = sha2::Sha256::new();

  sha256.write_all(&bytes)?;
  sha256.flush()?;

  let digest = sha256.finalize();

  Ok(base16ct::lower::encode_string(&digest))
}

fn output_watch_list<P>(path: P, extensions: &[&'static str]) -> Result<(), std::io::Error>
where
  P: AsRef<Path>,
{
  let dir = read_dir(path.as_ref())?;

  for entry in dir.flatten() {
    let metadata = entry.metadata()?;

    if metadata.is_file() {
      if let Some(ext) = entry.path().extension() {
        for allowed in extensions {
          if ext.eq_ignore_ascii_case(allowed) {
            println!("cargo::rerun-if-changed={:?}", entry.path());
            break;
          }
        }
      }
    } else if metadata.is_dir() {
      output_watch_list(entry.path(), extensions)?;
    }
  }

  Ok(())
}
