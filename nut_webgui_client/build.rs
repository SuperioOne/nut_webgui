use sha2::Digest;
use std::{
  fs::{File, canonicalize},
  io::{Read, Write},
  path::{Path, PathBuf},
  process::Command,
  str::FromStr,
};

fn main() -> Result<(), std::io::Error> {
  let outdir =
    std::env::var("OUT_DIR").expect("cargo did not set OUT_DIR env variable for build script.");

  let profile =
    std::env::var("PROFILE").expect("cargo did not set PROFILE env variable for build script.");

  if profile.eq_ignore_ascii_case("release") {
    Command::new("node")
      .args([
        "scripts/build.js",
        "--minify",
        format!("--outdir={}", &outdir).as_str(),
      ])
      .status()
      .unwrap();
  } else {
    Command::new("node")
      .args(["scripts/build.js", format!("--outdir={}", &outdir).as_str()])
      .status()
      .unwrap();
  }

  let outdir = PathBuf::from_str(&outdir).unwrap();
  create_asset(&outdir, "NUTWG_CLIENT_CSS", "style.css")?;
  create_asset(&outdir, "NUTWG_CLIENT_JS", "index.js")?;
  create_asset(&outdir, "NUTWG_CLIENT_ICON", "icon.svg")?;
  create_asset(&outdir, "NUTWG_CLIENT_SPRITE_SHEET", "feather-sprite.svg")?;

  println!("cargo::rerun-if-changed=src");
  println!("cargo::rerun-if-changed=static");
  println!("cargo::rerun-if-changed=../nut_webgui/src/http/hypermedia");
  println!("cargo::rerun-if-changed=package.json");

  Ok(())
}

fn create_asset(src_dir: &Path, env_prefix: &str, file_name: &str) -> Result<(), std::io::Error> {
  let src_dir = {
    if src_dir.is_relative() {
      canonicalize(src_dir)?
    } else {
      src_dir.to_path_buf()
    }
  };

  let file_path = src_dir.join(file_name);
  let mut content: Vec<u8> = Vec::new();
  _ = File::open(&file_path)?.read_to_end(&mut content)?;

  let sha256 = calc_sha256(&content)?;

  println!(
    "cargo::rustc-env={env_prefix}_PATH={}",
    file_path.to_str().expect("Not a valid unicode path string")
  );
  println!("cargo::rustc-env={env_prefix}_NAME={file_name}");
  println!("cargo::rustc-env={env_prefix}_SHA256={sha256}",);

  Ok(())
}

fn calc_sha256(bytes: &[u8]) -> Result<String, std::io::Error> {
  let mut sha256 = sha2::Sha256::new();

  sha256.write_all(bytes)?;
  sha256.flush()?;

  let digest = sha256.finalize();

  Ok(base16ct::lower::encode_string(&digest))
}
