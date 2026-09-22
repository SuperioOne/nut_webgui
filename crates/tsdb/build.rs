use std::{env, path::PathBuf};

fn main() {
  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  let src_path = PathBuf::from("./lmdb/libraries/liblmdb");

  cc::Build::new()
    .file(src_path.join("mdb.c"))
    .file(src_path.join("midl.c"))
    .file(src_path.join("module.c"))
    .flag_if_supported("-pthread")
    .compile("lmdb");

  let bindings = bindgen::Builder::default()
    .header("lmdb.h")
    .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
    .generate()
    .expect("Unable to generate lmdb bindings");

  bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("Couldn't write bindings!");

  println!("cargo::rerun-if-changed=build.rs");
  println!("cargo::rerun-if-changed=lmdb");
  println!("cargo::rerun-if-changed=lmdb.h");
}
