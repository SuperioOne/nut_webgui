# Building from source

## Linux:

### Requirements:
Required host tools are:
  - make
  - cargo
  - node
  - pnpm or npm
  - jq
  - coreutils/uutils, GNU gettext utilities

Optionally, host system should have gcc packages and Rust targets for cross-compilation.

Required packages:
  - riscv64-linux-gnu-gcc (RISC-V64)
  - aarch64-linux-gnu-gcc (AARCH64)
  - clang (ARMv6, ARMv7)

> Package names may differ between different distros.

Required Rust targets:
  - aarch64-unknown-linux-gnu
  - aarch64-unknown-linux-musl
  - arm-unknown-linux-musleabi
  - armv7-unknown-linux-musleabi
  - riscv64gc-unknown-linux-gnu
  - x86_64-unknown-linux-gnu
  - x86_64-unknown-linux-musl

Multi-Arch Container Images:
  - Qemu emulators for each target architecture.
  - Any OCI compliant image building tool of your choice (Buildah, Docker, Podman).

### Building Binaries

> To list all available recipes, use `make help`.

```bash
make build

# Output location ./bin/release/
```
or cross-compile everything (x86-64, AARCH64, ARMv7, ARMv6, RISC-V64)

```bash
make build-all

# Output location ./bin/<target-name>
```

### Generating dockerfiles

```bash
make gen-dockerfiles
# Output location ./bin/dockerfiles
```

# Development and Testing

1. Clone the git repository.

    ```shell
    git clone --recurse-submodules https://github.com/SuperioOne/nut_webgui.git
    ```

    > `--recurse-submodule` flag is required for the UPS validation tests. It pulls 
    > [NUT Device Dumps Library](https://github.com/networkupstools/nut-ddl) as submodule, 
    > which contains known UPS device dumps.

2. Run server
   - Start with bacon:

      `make watch` can start development server. It simply calls [bacon -j serve](https://github.com/Canop/bacon).

      ```bash
      # (Optional) Set your NUT test server configs.
      export NUTWG__CONFIG_FILE="test.config.toml"
      export NUTWG__LOG_LEVEL="trace"
      export NUTWG__UPSD__ADDRESS="10.0.0.1"
      export NUTWG__UPSD__USERNAME="cid"
      export NUTWG__UPSD__PASSWORD="i_am_atomic"

      make watch
      ```
    - or simply use Cargo: `cargo run -p nut_webgui`

## Tests

`make test` command runs all available tests.

## Simulating UPS devices and NUT server with containers

A basic NUT server container image is available at [tools/dummy_server](../tools/dummy_server) directory. 
It starts a NUT server, and automatically configures dummy UPS devices with the 
mounted `.seq` and `.dev` files.

> Expected file formats are:
> - <UPS_NAME>.dev
> - <UPS_NAME>.seq

```bash
cd tools/dummy_server

# Build dummy_server image
docker build -t dummy_server:latest -f dummy_server.Dockerfile

# Create a device dump directory to mount.
mkdir example-devices

# Copy your .seq and .dev files to the example-devices directory.

# Start NUT server with dummy devices
docker run --rm -p 3493:3493 -v "$(pwd)/example-devices":/nut_devices dummy_server:latest
```

For `dummy-ups` driver details, see [NetworkUpsTools - dummy-ups.](https://networkupstools.org/docs/man/dummy-ups.html)

For `.dev` and `.seq` details, see [nut-ddl.](https://github.com/networkupstools/nut-ddl)
