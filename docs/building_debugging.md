# Building code from source

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

### Development

You can start front-end and back-end server via `./start_dev.sh`. It simply calls [`cargo-watch`](https://github.com/watchexec/cargo-watch), esbuild and tailwind.

```shell
# Set your NUT server address
export UPSD_ADDR="10.0.0.0"

# (Optional) set username and password for testing INST_CMD.
export UPSD_USER="user_name"
export UPSD_PASS="yo"

./start_dev.sh
```

> Check available recipes inside the [Makefile](../Makefile).
