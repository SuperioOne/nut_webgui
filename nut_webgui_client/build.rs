use sha2::Digest;
use std::{
  fs::{File, read_dir},
  io::{Read, Write},
  path::Path,
  process::Command,
};

fn main() {
  // NOTE: Maybe I should ditch nodejs and simply use Tailwind rust crates + RsPack/FarmFe ???
  #[cfg(debug_assertions)]
  Command::new("node")
    .args(&["scripts/build.js", "--outdir=target/static"])
    .status()
    .unwrap();

  #[cfg(not(debug_assertions))]
  Command::new("node")
    .args(&["scripts/build.js", "--minify", "--outdir=target/static"])
    .status()
    .unwrap();

  println!("cargo::rustc-env=NUTWG_CLIENT_CSS_PATH=../target/static/style.css");
  println!("cargo::rustc-env=NUTWG_CLIENT_CSS_NAME=style.css");
  println!(
    "cargo::rustc-env=NUTWG_CLIENT_CSS_SHA256={sha256}",
    sha256 = calc_sha256("target/static/style.css").unwrap()
  );

  println!("cargo::rustc-env=NUTWG_CLIENT_JS_PATH=../target/static/index.js");
  println!("cargo::rustc-env=NUTWG_CLIENT_JS_NAME=index.js");
  println!(
    "cargo::rustc-env=NUTWG_CLIENT_JS_SHA256={sha256}",
    sha256 = calc_sha256("target/static/index.js").unwrap()
  );

  println!("cargo::rustc-env=NUTWG_CLIENT_ICON_PATH=../target/static/icon.svg");
  println!("cargo::rustc-env=NUTWG_CLIENT_ICON_NAME=icon.svg");
  println!(
    "cargo::rustc-env=NUTWG_CLIENT_ICON_SHA256={sha256}",
    sha256 = calc_sha256("target/static/icon.svg").unwrap()
  );

  println!("cargo::rustc-env=NUTWG_CLIENT_SPRITE_SHEET_PATH=../target/static/feather-sprite.svg");
  println!("cargo::rustc-env=NUTWG_CLIENT_SPRITE_SHEET_NAME=icon.svg");
  println!(
    "cargo::rustc-env=NUTWG_CLIENT_SPRITE_SHEET_SHA256={sha256}",
    sha256 = calc_sha256("target/static/feather-sprite.svg").unwrap()
  );

  output_watch_list("src", &["css", "js", "json", "rs", "svg"]).unwrap();
}

fn calc_sha256<P>(path: P) -> Result<String, std::io::Error>
where
  P: AsRef<Path>,
{
  let mut fs = File::open(path.as_ref())?;
  let mut sha256 = sha2::Sha256::new();
  let mut block = [0_u8; 4096];

  loop {
    match fs.read(&mut block)? {
      0 => break,
      read => {
        sha256.write_all(&block[..read])?;
      }
    }
  }

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
