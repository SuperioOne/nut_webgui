use sha2::Digest;
use std::{
  fs::{File, read_dir},
  io::{Read, Write},
  path::{Path, PathBuf},
  process::Command,
  str::FromStr,
};

fn main() {
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

  let css_path = target.join("style.css");
  println!(
    "cargo::rustc-env=NUTWG_CLIENT_CSS_PATH={}",
    PathBuf::from_str("..")
      .unwrap()
      .join(&css_path)
      .to_str()
      .unwrap()
  );
  println!("cargo::rustc-env=NUTWG_CLIENT_CSS_NAME=style.css");
  println!(
    "cargo::rustc-env=NUTWG_CLIENT_CSS_SHA256={sha256}",
    sha256 = calc_sha256(css_path).unwrap()
  );

  let js_path = target.join("index.js");
  println!(
    "cargo::rustc-env=NUTWG_CLIENT_JS_PATH={}",
    PathBuf::from_str("..")
      .unwrap()
      .join(&js_path)
      .to_str()
      .unwrap()
  );
  println!("cargo::rustc-env=NUTWG_CLIENT_JS_NAME=index.js");
  println!(
    "cargo::rustc-env=NUTWG_CLIENT_JS_SHA256={sha256}",
    sha256 = calc_sha256(js_path).unwrap()
  );

  let icon_path = target.join("icon.svg");
  println!(
    "cargo::rustc-env=NUTWG_CLIENT_ICON_PATH={}",
    PathBuf::from_str("..")
      .unwrap()
      .join(&icon_path)
      .to_str()
      .unwrap()
  );
  println!("cargo::rustc-env=NUTWG_CLIENT_ICON_NAME=icon.svg");
  println!(
    "cargo::rustc-env=NUTWG_CLIENT_ICON_SHA256={sha256}",
    sha256 = calc_sha256(icon_path).unwrap()
  );

  let sprite_sheet_path = target.join("feather-sprite.svg");
  println!(
    "cargo::rustc-env=NUTWG_CLIENT_SPRITE_SHEET_PATH={}",
    PathBuf::from_str("..")
      .unwrap()
      .join(&sprite_sheet_path)
      .to_str()
      .unwrap()
  );
  println!("cargo::rustc-env=NUTWG_CLIENT_SPRITE_SHEET_NAME=.feather-sprite.svg");
  println!(
    "cargo::rustc-env=NUTWG_CLIENT_SPRITE_SHEET_SHA256={sha256}",
    sha256 = calc_sha256(sprite_sheet_path).unwrap()
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
