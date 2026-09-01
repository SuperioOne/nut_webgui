# Binary Installation

`nut_webgui` installation only requires copying files, since it's a
self-contained application and it does not require any additional runtime
dependency. The only exceptions are `*-gnu` targets, which link with `glibc`.

For other UNIX-based operating systems, there are currently no pre-built
packages, and it must be built from the source.

## Method 1: install.sh script

`install.sh` script automatically detects the system configuration and installs
`nut_webgui`. It also creates empty `/etc/nut_webgui/config.toml`,
`/etc/nut_webgui/server.key`, man pages, and service files if they're not
already present on the system.

> [!TIP]
> `install.sh` script can be used for version upgrades. The installation process
> skips any existing config files while upgrading the binary, man pages, and
> service files.

> [!NOTE]
> For x86_64, please note that the install script does not check CPU flags to
> detect micro-architecture levels. See the `Optional parameters` section if you
> want to take advantage of modern CPU features such as AVX-512.

### Usage
1. Download
   ```sh
   curl -fL -o install.sh \
   "https://codeberg.org/SuperiorOne/nut_webgui/releases/download/latest/install.sh"
   ```
2. **REVIEW**
   ```sh
   less ./install.sh
   ```

3. Run
   ```sh
   chmod +x ./install.sh
   sh ./install.sh
   ```

Or pipe the curl output directly into `sh` (* *Insert Michael worried meme here* *):

```sh
curl -sfL \
"https://codeberg.org/SuperiorOne/nut_webgui/releases/download/latest/install.sh" | sh -
```

### Optional parameters

`install.sh` has optional parameters that allow users to override
auto-detection, install location, server key, etc.

**Example usage**
```sh
DEBUG=1 \
INTERACTIVE="false" \
INIT_SYSTEM="runit" \
MIRROR_NAME="codeberg" \
NUTWG_TARGET="x86-64-v3-musl" \
NUTWG_SERVER_KEY="SuperSecretKey" \
INSTALL_PREFIX="/home/test/my/chroot" \
./install.sh
```

#### NUTWG_TARGET
Override the detected libc and CPU architecture with the given value.

Available Linux targets are:
- aarch64-gnu
- aarch64-musl
- armv6-musleabi
- armv7-musleabi
- riscv64gc-gnu
- riscv64gc-musl
- x86-64-gnu
- x86-64-musl
- x86-64-v3-gnu
- x86-64-v3-musl
- x86-64-v4-gnu
- x86-64-v4-musl

#### NUTWG_SERVER_KEY
Set the server key to the given value. If not defined, the install script
generates a random key.

#### INIT_SYSTEM
Override the detected init system with the given value.

Available init system options are:
- openrc
- runit
- systemd

#### INTERACTIVE
The script displays a basic installation plan and asks for user confirmation
before making actual changes to the filesystem. For automated installations,
this confirmation step can be skipped by setting `INTERACTIVE=false`.

#### MIRROR_NAME
Change the package mirror. Default value is `github`.

Available mirror names are:
- codeberg
- github

#### INSTALL_PREFIX
Change the install prefix. Mainly intended for jailed directories and testing
purposes.

#### DEBUG
When `DEBUG=1`, it enables `set -ex` and shows executed shell commands.

## Method 2: From source code

`nut_webgui` can be built and installed directly from the source code.

> [!WARNING]
> This installation method only installs the binary itself, and it does not
> create any additional service or config files.

**Prerequisites:**
   - cargo
   - git
   - make
   - node
   - npm or pnpm *(pick your poison)*
   - rust toolchain

### make

#### System: `/usr/local/bin`

```sh
git clone --depth=1 "https://codeberg.org/SuperiorOne/nut_webgui.git"
cd nut_webgui
make build
sudo make install
```

#### User: `$HOME/.local/bin`

```sh
git clone --depth=1 "https://codeberg.org/SuperiorOne/nut_webgui.git"
cd nut_webgui
make build
make install INSTALL_PREFIX="$HOME/.local/bin"
```

### cargo

Alternatively, it can be installed via `cargo` (`$HOME/.cargo/bin`).

```sh
cargo install --git "https://codeberg.org/SuperiorOne/nut_webgui.git"
```

## Method 3: Extracting tar archive

You can simply download and extract the tar archive from the
[releases page](https://codeberg.org/SuperiorOne/nut_webgui/releases) to
wherever you want. Each tar archive contains: the server executable, config
templates, man pages, and example service files for the supported init systems.

## (Optional) Enabling services

Current packages are bundled with basic service files for the different init
systems. If you prefer to use environment variables instead of the `config.toml`
file, each service setup also comes with its own environment config file.

### runit

```sh
ln -s /etc/sv/nut_webgui /var/service
```

**Service config file:** `/etc/sv/nut_webgui/conf`

### OpenRC

```sh
rc-update add nut_webgui
rc-service nut_webgui start
```

**Service config file:** `/etc/init.d/nut_webgui`

### systemd

```sh
systemctl daemon-reload
systemctl enable --now nut_webgui.service
```

**Service config file:** `/etc/nut_webgui/nut_webgui.env`

## Uninstalling

Simply remove the executable and config files:

```sh
rm /usr/local/bin/nut_webgui
rm -r /etc/nut_webgui
rm -r /usr/local/share/nut_webgui
```

systemd

```sh
systemctl stop nut_webgui.service
systemctl disable nut_webgui.service
rm /etc/systemd/system/nut_webgui.service
```

OpenRC

```sh
rc-service nut_webgui stop
rc-update del nut_webgui
rm /etc/init.d/nut_webgui
rm /etc/conf.d/nut_webgui
```

runit

```sh
rm /var/service/nut_webgui
rm -r /etc/sv/nut_webgui
```
