use sha2::Digest;
use std::{
  fs::{File, canonicalize, copy},
  io::{Read, Write},
  path::{Path, PathBuf},
  str::FromStr,
};

trait IntoCliArg {
  fn into_cli_arg(&self) -> Option<&str>;
}

impl IntoCliArg for &str {
  #[inline]
  fn into_cli_arg(&self) -> Option<&str> {
    Some(self)
  }
}

impl IntoCliArg for String {
  #[inline]
  fn into_cli_arg(&self) -> Option<&str> {
    Some(self.as_str())
  }
}

impl<T> IntoCliArg for Option<T>
where
  T: AsRef<str>,
{
  #[inline]
  fn into_cli_arg(&self) -> Option<&str> {
    self.as_ref().map(|v| v.as_ref())
  }
}

macro_rules! exec {
  ($cmd:literal) => {
    std::process::Command::new($cmd)
      .status()
      .inspect(|status| {
        if !status.success() {
          println!("cargo::error={}: execution failed.", $cmd);
        }
      })
      .inspect_err(|err| {
        println!("cargo::error={}", err.to_string());
      });
  };

  ($cmd:literal, $($arg:expr),+) => {
    {
      let mut args: Vec<&str> = Vec::new();

      $(
        if let Some(arg) = $arg.into_cli_arg() {
          args.push(arg);
        }
      )+

      std::process::Command::new($cmd)
        .args(&args)
        .status()
        .inspect(|status| {
          if !status.success() {
            println!("cargo::error={}: execution failed.", $cmd);
          }
        })
        .inspect_err(|err| {
          println!("cargo::error={}", err.to_string());
        })
    }
  };
}

fn main() -> Result<(), std::io::Error> {
  let outdir =
    std::env::var("OUT_DIR").expect("cargo did not set OUT_DIR env variable for build script.");
  let outdir = PathBuf::from_str(&outdir).unwrap();

  let profile =
    std::env::var("PROFILE").expect("cargo did not set PROFILE env variable for build script.");

  match exec!("node", "--version") {
    Ok(status) if status.success() => {}
    _ => {
      println!(
        "cargo::error=node is required for building the client assets. Make sure the system has nodejs installed."
      );
      return Ok(());
    }
  }

  match detect_package_manager() {
    Some(PackageManager::Npm) => exec!("npm", "install")?,
    Some(PackageManager::Pnpm) => exec!("pnpm", "install")?,
    None => {
      println!("cargo::error=npm or pnpm is required for initializing node_modules directory.");
      return Ok(());
    }
  };

  let minify: Option<&'static str> = if profile.eq_ignore_ascii_case("release") {
    Some("--minify")
  } else {
    None
  };

  let style_css_path = format!("{}", outdir.join("style.css").display());
  exec!(
    "node",
    "node_modules/@tailwindcss/cli/dist/index.mjs",
    "-i",
    "src/style.css",
    "-o",
    style_css_path,
    minify
  )?;

  let outdir_arg = format!("--outdir={}", outdir.display());
  exec!(
    "node",
    "node_modules/esbuild/bin/esbuild",
    "src/index.js",
    "--bundle",
    "--format=iife",
    "--target=firefox109,chrome108,safari15",
    outdir_arg,
    minify
  )?;

  copy("static/icon.svg", outdir.join("icon.svg"))?;
  copy(
    "static/feather-sprite.svg",
    outdir.join("feather-sprite.svg"),
  )?;

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

enum PackageManager {
  Pnpm,
  Npm,
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
