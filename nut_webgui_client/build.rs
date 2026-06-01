use rolldown::{
  Bundler, BundlerOptions, BundlerTransformOptions, CommentsOptions, OptimizationOption,
  OutputFormat, Platform,
};
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

#[tokio::main]
async fn main() {
  if let Err(err) = bundle().await {
    println!("cargo::error=client asset bundler failed");
    println!("cargo::error={}", err);
  }
}

async fn bundle() -> Result<(), Box<dyn core::error::Error>> {
  let srcdir = PathBuf::from_str("./src/")?.canonicalize()?;
  let outdir = std::env::var("OUT_DIR")?;
  let profile = std::env::var("PROFILE")?;
  let minify: Option<&'static str> = if profile.eq_ignore_ascii_case("release") {
    Some("--minify")
  } else {
    None
  };

  match detect_package_manager() {
    Some(PackageManager::Npm) => exec!("npm", "install")?,
    Some(PackageManager::Pnpm) => exec!("pnpm", "install")?,
    None => {
      println!("cargo::error=npm or pnpm is required for initializing node_modules directory.");
      return Err(io::Error::from(io::ErrorKind::NotFound).into());
    }
  };

  let mut js_bundler = Bundler::new(BundlerOptions {
    comments: Some(CommentsOptions {
      jsdoc: false,
      annotation: false,
      legal: true,
    }),
    minify: Some(rolldown::RawMinifyOptions::Bool(minify.is_some())),
    treeshake: rolldown::TreeshakeOptions::Boolean(true),
    minify_internal_exports: Some(true),
    optimization: Some(OptimizationOption {
      inline_const: Some(rolldown::InlineConstOption::Bool(false)),
      ..Default::default()
    }),
    polyfill_require: Some(false),
    format: Some(OutputFormat::Iife),
    platform: Some(Platform::Browser),
    input: Some(vec!["./index.js".to_owned().into()]),
    cwd: Some(srcdir),
    clean_dir: Some(false),
    dir: Some(outdir.clone()),
    transform: Some(BundlerTransformOptions {
      target: Some(rolldown::Either::Left("es2022".to_owned())),
      ..Default::default()
    }),
    ..Default::default()
  })?;

  exec!("node", "--version").inspect_err(|_| {
    println!(
      "cargo::error=node is required for building the client assets. Make sure the system has nodejs installed."
    );
  })?;

  let outdir = PathBuf::from_str(&outdir)?;

  js_bundler.write().await?;
  exec!("node", "./postcss.build.js", outdir.join("style.css"))?;
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
  println!("cargo::rerun-if-changed=postcss.build.js");
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
  let sha256 = calc_sha256(&content);

  println!(
    "cargo::rustc-env={env_prefix}_PATH={}",
    file_path.to_str().expect("Not a valid unicode path string")
  );
  println!("cargo::rustc-env={env_prefix}_NAME={file_name}");
  println!("cargo::rustc-env={env_prefix}_SHA256={sha256}",);

  Ok(())
}

fn calc_sha256(bytes: &[u8]) -> String {
  let mut sha256 = sha2::Sha256::new();
  sha256.update(bytes);
  let digest = sha256.finalize();

  base16ct::lower::encode_string(&digest)
}

impl core::fmt::Display for CommandError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "{}: {}", self.name, self.error)
  }
}

impl core::error::Error for CommandError {}
