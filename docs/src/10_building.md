# Building from source

## Requirements:
Required host tools are:
  - make
  - cargo
  - rust >= 1.95
  - node
  - pnpm or npm
  - jq
  - GNU gettext utilities

## Building binary

Basic usage of `make`:

```sh
make build

./bin/artifact/release/nut_webgui --help
```

`cargo` can also be used directly:

```sh
cargo build --release -p nut_webgui

./target/release/nut_webgui --help
```

## Cross building

In order to compile the application for different architectures, the host system
should have gcc, LLVM, and Rust targets for cross-compilation.

Required host tools:
  - riscv64-linux-gnu-gcc
  - riscv64-linux-musl-gcc
  - aarch64-linux-gnu-gcc
  - llvm/clang

Required Rust targets:
  - aarch64-unknown-linux-gnu
  - aarch64-unknown-linux-musl
  - arm-unknown-linux-musleabi
  - armv7-unknown-linux-musleabi
  - riscv64gc-unknown-linux-gnu
  - riscv64gc-unknown-linux-musl
  - x86_64-unknown-linux-gnu
  - x86_64-unknown-linux-musl

Cross-building is disabled by default. It can be enabled by the `configure.sh`
script.

```sh
./configure.sh --enable-all
make build-all
ls ./bin/artifact
```

Optionally, built binaries can be packed into tar files with:

```sh
make package
ls ./bin/package
```

> [!TIP]
> Run `./configure.sh --help` to see all available options.

## Building Multi-Arch container images

Building multi-arch container images requires `buildah`, `qemu`, and
cross-building setup for `*-musl` targets.

Example setup for arm64/v8, amd64, riscv64, and arm/v7:

```sh
./configure \
  --enable-container-image \
  --enable-aarch64-musl \
  --enable-armv7-musleabi \
  --enable-riscv64gc-musl \
  --enable-x86-64-musl

make build-images
buildah images
```
