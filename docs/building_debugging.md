# Building from source

## Linux:

### Requirements:
Required host tools are:
  - make
  - cargo
  - node
  - pnpm
  - jq

Optionally, You need the following packages and Rust targets for cross-compilation.

Required packages:
  - riscv64-linux-gnu-gcc
  - aarch64-linux-gnu-gcc

Required Rust targets:
  - aarch64-unknown-linux-gnu
  - aarch64-unknown-linux-musl
  - arm-unknown-linux-musleabi
  - armv7-unknown-linux-musleabi
  - riscv64gc-unknown-linux-gnu
  - x86_64-unknown-linux-gnu
  - x86_64-unknown-linux-musl

### Building

```shell
make build

# Output dirs ./bin/release/ and ./bin/static
```
or cross-compile everything (x86-64, ARM64-v8, ARM7, ARM6, RISC-V64)

```shell
make build-all

# Output dirs ./bin/<target-name> and ./bin/static
```

> For more options, check the available recipes inside the [Makefile](../Makefile).

# Development and Testing

Clone the git repository.

```shell
git clone --recurse-submodules https://github.com/SuperioOne/nut_webgui.git
```

> `--recurse-submodule` flag is required for the UPS validation tests. It pulls 
> [NUT Device Dumps Library](https://github.com/networkupstools/nut-ddl) as submodule, 
> which contains known UPS device dumps.

You can start front-end and back-end server via `./start_dev.sh`. It simply calls
[`cargo-watch`](https://github.com/watchexec/cargo-watch), esbuild and tailwind.

```shell
# (Optional) Set your NUT server address, default is localhost.
export UPSD_ADDR="10.0.0.1"

# (Optional) set username and password to test INST_CMD.
export UPSD_USER="user_name"
export UPSD_PASS="yo"

./start_dev.sh
```

## Testing

Simply use `make test` command to run all available tests.

## Simulating UPS devices with container

A basic NUT server container image is available at [tools/dummy_server](../tools/dummy_server) directory. 
It starts a NUT server, and automatically configures dummy UPS devices with the 
mounted `.seq` and `.dev` files.

> Expected file formats are:
> - <UPS_NAME>.dev
> - <UPS_NAME>.seq

```shell
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

