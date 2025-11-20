# Binary Installation

## Method 1: install.sh script

`install.sh` script automatically detects the system configuration and installs `nut_webgui`.
It also creates empty `/etc/nut_webgui/config.toml` file and systemd `nut_webgui.service`
if they're not already present on the system.

> For x86_64, please note that install script does not check CPU flags to detect
> micro-architecture levels. See `Custom Target` section if you want to take advantage of
> the modern features such as AVX512.

### Steps
1. Download
   ```shell
   curl -sfL https://github.com/SuperioOne/nut_webgui/releases/download/v0.7.0/install.sh -o install.sh
   ```
2. **REVIEW**
   ```shell
   less ./install.sh
   ```

3. Run
   ```shell
   sh ./install.sh
   ```


Or pipe curl output into `sh` directly (* *Insert Michael worried meme here* *):
 ```shell
curl -sfL https://github.com/SuperioOne/nut_webgui/releases/download/v0.7.0/install.sh | sh -
```

### (Optional) Custom target

   Use `NUTWG_TARGET` environment variable to specify the target if needed:

   ```shell
   curl -sfL https://github.com/SuperioOne/nut_webgui/releases/download/v0.7.0/install.sh | NUTWG_TARGET="x86-64-v3-musl" sh -
   ```

   Available targets are:
   - aarch64-gnu
   - aarch64-musl
   - armv6-musleabi
   - armv7-musleabi
   - riscv64gc-gnu
   - x86-64-gnu
   - x86-64-musl
   - x86-64-v3-gnu
   - x86-64-v3-musl
   - x86-64-v4-gnu
   - x86-64-v4-musl

### Uninstalling

Stop the service if `systemd` is enabled:

```shell
systemctl stop nut_webgui.service
systemctl disable nut_webgui.service
```

Simply remove the executable and config files:

```shell
rm /usr/local/bin/nut_webgui
rm /etc/systemd/system/nut_webgui.service
rm -r /etc/nut_webgui
```

## Method 2: From source code

`nut_webgui` can be build and install directly from the source code.

> This installation method does not create systemd service or empty config.toml file.

**Prerequisites:**
   - cargo
   - git
   - make
   - jq
   - node
   - npm or pnpm
   - rust toolchain

### make

Makefile has two different recipe for installation:

   1. System-wide installation (`/usr/local/bin`):
   ```shell
   git clone --depth=1 https://github.com/SuperioOne/nut_webgui.git
   cd nut_webgui
   make install
   ```

   2. User only installation (`$HOME/.local/bin`):
   ```shell
   git clone --depth=1 https://github.com/SuperioOne/nut_webgui.git
   cd nut_webgui
   make install-local
   ```

### cargo

Alternatively, it can be installed via `cargo` (`$HOME/.cargo/bin`).

```
cargo install --git https://github.com/SuperioOne/nut_webgui.git
```

## Method 3: I use Arch Linux btw™

A `PKGBUILD` file is available on the [releases page](https://github.com/SuperioOne/nut_webgui/releases). 
Similar to `install.sh`, it creates `systemd` service and empty config file.

> Package is currently not available on AUR.

```shell
curl -sfL https://github.com/SuperioOne/nut_webgui/releases/download/v0.7.0/PKGBUILD -o PKGBUILD
makepkg -i
```

## Method 4: Extracting tar archive

`nut_webgui` is only a single executable and does not require any runtime dependency*.
You can simply download and extract the tar archive from the [releases page](https://github.com/SuperioOne/nut_webgui/releases) to anywhere you want.

> `*` The only exceptions are `*-gnu` targets, they link with `glibc`. This is only relevant if you use a distro with different libc implementation such as Alpine Linux.

