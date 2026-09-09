use base64::{Engine, prelude::BASE64_STANDARD};
use sha2::Digest;
use std::{
  fs::{File, canonicalize, copy},
  io::{self, Read},
  path::{Path, PathBuf},
  str::FromStr,
};

#[derive(Clone, Copy)]
enum PackageManager {
  Pnpm,
  Npm,
}

#[derive(Debug)]
struct CommandError {
  name: &'static str,
  error: std::io::Error,
}

struct Hashes {
  sha256: String,
  short_hash: String,
}

macro_rules! exec {
  ($cmd:literal) => {
      exec!(@internal std::process::Command::new($cmd), cmd);
  };

  ($cmd:literal, $($arg:expr),+) => {
    {
      let mut cmd = std::process::Command::new($cmd);
      $( _ = cmd.arg($arg); )+
      exec!(@internal $cmd, cmd)
    }
  };

  (@internal $name:literal, $cmd:expr) => {
      match $cmd.status() {
        Ok(status) => {
          if status.success() {
            Ok(status)
          }
          else {
            Err(CommandError {
              name: $name,
              error: std::io::Error::new(std::io::ErrorKind::Other, "command execution failed"),
            })
          }
        },
        Err(err) => {
          Err(CommandError {
            name: $name,
            error: err
          })
        }
      }
  };
}

fn main() {
  if let Err(err) = bundle() {
    println!("cargo::error=client asset bundler failed");
    println!("cargo::error={}", err);
  }
}

fn bundle() -> Result<(), Box<dyn core::error::Error>> {
  let srcdir = PathBuf::from_str("./src/")?.canonicalize()?;
  let outdir = std::env::var("OUT_DIR")?;
  let profile = std::env::var("PROFILE")?;
  let minify = profile.eq_ignore_ascii_case("release").to_string();
  let outdir = PathBuf::from_str(&outdir)?;

  match detect_package_manager() {
    Some(PackageManager::Npm) => exec!("npm", "install")?,
    Some(PackageManager::Pnpm) => exec!("pnpm", "install")?,
    None => {
      println!("cargo::error=npm or pnpm is required for initializing node_modules directory.");
      return Err(io::Error::from(io::ErrorKind::NotFound).into());
    }
  };

  exec!("node", "--version").inspect_err(|_| {
    println!(
      "cargo::error=node is required for building the client assets. Make sure the system has nodejs installed."
    );
  })?;

  exec!(
    "node",
    "./build.js",
    "bundle-js",
    "-i",
    srcdir.join("index.js"),
    "-o",
    outdir.join("index.js"),
    "--minify",
    &minify
  )?;
  exec!(
    "node",
    "./build.js",
    "bundle-css",
    "-i",
    srcdir.join("style.css"),
    "-o",
    outdir.join("style.css"),
    "--minify",
    &minify
  )?;
  copy("./static/icon.svg", outdir.join("icon.svg"))?;
  copy(
    "./static/feather-sprite.svg",
    outdir.join("feather-sprite.svg"),
  )?;

  create_asset(&outdir, "NUTWG_CLIENT_CSS", "style.css")?;
  create_asset(&outdir, "NUTWG_CLIENT_JS", "index.js")?;
  create_asset(&outdir, "NUTWG_CLIENT_ICON", "icon.svg")?;
  create_asset(&outdir, "NUTWG_CLIENT_SPRITE_SHEET", "feather-sprite.svg")?;

  println!("cargo::rerun-if-changed=src");
  println!("cargo::rerun-if-changed=static");
  println!("cargo::rerun-if-changed=build.js");
  println!("cargo::rerun-if-changed=build.rs");
  println!("cargo::rerun-if-changed=package.json");

  Ok(())
}

fn detect_package_manager() -> Option<PackageManager> {
  if let Ok(status) = exec!("pnpm", "--version")
    && status.success()
  {
    Some(PackageManager::Pnpm)
  } else if let Ok(status) = exec!("npm", "--version")
    && status.success()
  {
    Some(PackageManager::Npm)
  } else {
    None
  }
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
  let hashes = calc_hashes(&content);

  println!(
    "cargo::rustc-env={env_prefix}_PATH={}",
    file_path.to_str().expect("Not a valid unicode path string")
  );
  println!("cargo::rustc-env={env_prefix}_NAME={file_name}");
  println!("cargo::rustc-env={env_prefix}_SHA256={}", hashes.sha256);
  println!("cargo::rustc-env={env_prefix}_HASH={}", hashes.short_hash);

  Ok(())
}

fn calc_hashes(bytes: &[u8]) -> Hashes {
  let mut sha256 = sha2::Sha256::new();
  sha256.update(bytes);
  let digest = sha256.finalize();

  Hashes {
    sha256: BASE64_STANDARD.encode(&digest),
    short_hash: base16ct::lower::encode_string(&digest)[0..8].to_owned(),
  }
}

impl core::fmt::Display for CommandError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "{}: {}", self.name, self.error)
  }
}

impl core::error::Error for CommandError {}
