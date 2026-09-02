#!/usr/bin/env sh
if [ "$DEBUG" = "1" ]; then
    set -ex
else
    set -e
fi

# Inspired from the first version of k3s install.sh
# @ibuildthecloud

is_enabled() {
    case "$1" in
        y | Y | 1 | true | True | TRUE)
            echo 1
            ;;
        n | N | 0 | false | False | FALSE)
            echo 0
            ;;
        *)
            echo "err: Unknown boolean flag. Check your parameters"
            ;;
    esac
}

confirm_prompt() {
    if [ $(is_enabled "$INTERACTIVE") = 1 ]; then
        local MESSAGE CONFIRMATION
        MESSAGE="$1"
        CONFIRMATION=

        while true; do
            read -p "$MESSAGE [y/N] :" CONFIRMATION </dev/tty
            case "$CONFIRMATION" in
                y | Y | 1 | true | True | TRUE)
                    echo 1
                    break
                    ;;
                n | N | 0 | false | False | FALSE)
                    echo 0
                    break
                    ;;
                *)
                    continue
                    ;;
            esac
        done
    else
        echo 1
    fi
}

detect_init_system() {
    local INIT_SYSTEM_PATH
    if [ -e "/sbin/init" ]; then
        INIT_SYSTEM_PATH="$(realpath "/sbin/init")"
        INIT_SYSTEM="$(basename "$INIT_SYSTEM_PATH")"

        case "$INIT_SYSTEM" in
            systemd)
                INIT_SYSTEM="systemd"
                ;;
            runit-init)
                if [ -d "/etc/sv" ]; then
                    INIT_SYSTEM="runit"
                fi
                ;;
            *)
                if [ -e "/sbin/openrc-run" ]; then
                    INIT_SYSTEM="openrc"
                else
                    INIT_SYSTEM=
                fi
                ;;
        esac
    fi
}

detect_libc() {
    if getconf "GNU_LIBC_VERSION" 2>&1 > /dev/null; then
        NUTWG_LIBC_TYPE="gnu"
    else
        NUTWG_LIBC_TYPE="musl"
    fi
}

detect_target() {
    if [ -z "$ARCH" ]; then
        NUTWG_ARCH="$(uname -m)"
    else
        NUTWG_ARCH="$ARCH"
    fi

    if [ "$NUTWG_ARCH" = "amd64" ]; then
        NUTWG_ARCH="x86_64"
    elif [ "$NUTWG_ARCH" = "arm64" ]; then
        NUTWG_ARCH="aarch64"
    fi

    case "$NUTWG_ARCH" in
        x86_64)
            if [ "$NUTWG_LIBC_TYPE" = "gnu" ]; then
                NUTWG_TARGET="x86-64-gnu"
            else
                NUTWG_TARGET="x86-64-musl"
            fi
            ;;
        aarch64)
            if [ "$NUTWG_LIBC_TYPE" = "gnu" ]; then
                NUTWG_TARGET="aarch64-gnu"
            else
                NUTWG_TARGET="aarch64-musl"
            fi
            ;;
        riscv64)
            if [ "$NUTWG_LIBC_TYPE" = "gnu" ]; then
                NUTWG_TARGET="riscv64gc-gnu"
            else
                NUTWG_TARGET="riscv64gc-musl"
            fi
            ;;
        armv7)
            NUTWG_TARGET="armv7-musleabi"
            ;;
        armv6)
            NUTWG_TARGET="armv6-musleabi"
            ;;
        *)
            echo "err: Unsupported CPU architecture: $ARCH"
            echo "If you think script detection is incorrect, try setting NUTWG_TARGET env variable with the one of the following options:"
            echo "  aarch64-gnu"
            echo "  aarch64-musl"
            echo "  armv6-musleabi"
            echo "  armv7-musleabi"
            echo "  riscv64gc-gnu"
            echo "  riscv64gc-musl"
            echo "  x86-64-gnu"
            echo "  x86-64-musl"
            echo "  x86-64-v3-gnu"
            echo "  x86-64-v3-musl"
            echo "  x86-64-v4-gnu"
            echo "  x86-64-v4-musl"
            exit 1
            ;;
    esac
}

detect_download_client() {
    if command -v curl 2>&1 > /dev/null; then
        DOWNLOAD_CLIENT=curl
    elif command -v wget 2>&1 > /dev/null; then
        DOWNLOAD_CLIENT=wget
    else
        echo "err: Install script requires curl or wget to download binaries."
        exit 1;
    fi
}

download() {
    case "$DOWNLOAD_CLIENT" in
        curl)
            curl -fL --progress-bar --output-dir "$2" -O "$1"
            ;;
        wget)
            wget --backups=3 -q --show-progress --directory-prefix="$2" "$1"
            ;;
        *)
            echo "err: No download client is set."
            exit 1
            ;;
    esac
}

manifest_get_target() {
    local MANIFEST_PATH TARGET
    MANIFEST_PATH="$1"
    TARGET="$2"

    cat "$MANIFEST_PATH" | awk \
        -v "TARGET=$TARGET" \
        -v 'RS=---\n' \
        -v 'FS=\n' \
    '{
          for(i = 0; i < NF; i++) {
              s = index($i,":")
              if (substr($i,0,s-1) == "Target") {
                  val = substr($i,s+1)
                  gsub(/^[[:space:]]+|[[:space:]]+$/, "", val)
                  if(val == TARGET) {
                      gsub(/^[[:space:]]+|[[:space:]]+$/, "", $0)
                      print $0
                  }
               }
          }
     }'
}

get_field() {
    local CONTENT FIELD_NAME
    CONTENT="$1"
    FIELD_NAME="$2"

    echo "$CONTENT" | awk \
        -v "FIELD_NAME=$FIELD_NAME" \
    '{
           s = index($i,":")
           if (substr($i,0,s-1) == FIELD_NAME) {
               val = substr($i,s+1)
               gsub(/^[[:space:]]+|[[:space:]]+$/, "", val)
               print val
            }
     }'
}

INIT_SYSTEM="${INIT_SYSTEM:-""}"
INSTALL_PREFIX="${INSTALL_PREFIX:-""}"
MIRROR_NAME="${MIRROR_NAME:-"github"}"
INTERACTIVE="${INTERACTIVE:-"1"}"
NUTWG_SERVER_KEY="${NUTWG_SERVER_KEY:-""}"
NUTWG_TARGET="${NUTWG_TARGET:-""}"

INSTALL_CONFIG_DIR="$INSTALL_PREFIX/etc/nut_webgui"
INSTALL_BIN_DIR="$INSTALL_PREFIX/usr/local/bin"
INSTALL_MAN_DIR="$INSTALL_PREFIX/usr/local/share/man"
INSTALL_TEMPLATE_DIR="$INSTALL_PREFIX/usr/local/share/nut_webgui"
MIRROR_URI=
SUDO=

case "$MIRROR_NAME" in
    github)
        MIRROR_URI="https://github.com/SuperioOne/nut_webgui/releases/latest/download"
        ;;
    codeberg)
        MIRROR_URI="https://codeberg.org/SuperiorOne/nut_webgui/releases/download/latest"
        ;;
    *)
        echo "err: Unknown mirror name. Currently only github and codeberg is supported."
        exit 1
        ;;
esac

if [ "$(id -u)" -ne 0 ]; then
    if command -v sudo 2>&1 > /dev/null; then
        SUDO=sudo
    elif command -v doas 2>&1 > /dev/null; then
        SUDO=doas
    else
        echo "Neither sudo or doas exists on the system. Try running script with elevated permissions."
        exit 1;
    fi
fi

if [ -z "$NUTWG_TARGET" ]; then
    detect_libc
    detect_target
fi

if [ -z "$INIT_SYSTEM" ]; then
    detect_init_system
fi

detect_download_client

echo "INFO"
echo "  Mirror URI           : $MIRROR_URI"
echo "  Target               : $NUTWG_TARGET"
echo "  Init system          : $INIT_SYSTEM"
echo "PATHS"
echo "  Server binary dir    : $INSTALL_BIN_DIR"
echo "  config.toml path     : $INSTALL_CONFIG_DIR/config.toml"
echo "  server.key path      : $INSTALL_CONFIG_DIR/server.key"
echo "  users.toml path      : $INSTALL_CONFIG_DIR/users.toml"
echo "  Config templates dir : $INSTALL_TEMPLATE_DIR"
echo "  Man pages dir        : $INSTALL_MAN_DIR"
echo ""

if [ $(confirm_prompt "Do you confirm the installation?") != 1 ]; then
    exit 1;
fi

TEMP_DIR="$(mktemp -d "/tmp/nut_webgui.XXXXXX")"

echo "info: Downloading $MIRROR_URI/MANIFEST"
download "$MIRROR_URI/MANIFEST" "$TEMP_DIR"

TARGET_INFO="$(manifest_get_target "$TEMP_DIR/MANIFEST" "$NUTWG_TARGET")"

if [ -z "$TARGET_INFO" ]; then
    echo "err: Manifest file does not contain target: $NUTWG_TARGET"
    exit 1
fi

PKG_SHA256="$(get_field "$TARGET_INFO" "SHA256")"
PKG_FILENAME="$(get_field "$TARGET_INFO" "Filename")"
PKG_NAME="$(get_field "$TARGET_INFO" "Fullname")"
PKG_TAR_PATH="$TEMP_DIR/$PKG_FILENAME"
PKG_PATH="$TEMP_DIR/$PKG_NAME"

echo "info: Downloading $MIRROR_URI/$PKG_FILENAME"
download "$MIRROR_URI/$PKG_FILENAME" "$TEMP_DIR"

if echo "$PKG_SHA256 $PKG_TAR_PATH" | sha256sum -c ; then
    tar -xf "$PKG_TAR_PATH" -C "$TEMP_DIR"
else
    echo "err: $PKG_FILENAME sha256 verification failed."
    exit 1;
fi

$SUDO install -D -m 755 "$PKG_PATH/nut_webgui" "$INSTALL_BIN_DIR/nut_webgui"
$SUDO install -D -m 644 "$PKG_PATH/config.toml" "$INSTALL_TEMPLATE_DIR/config.toml"
$SUDO install -D -m 644 "$PKG_PATH/users.toml" "$INSTALL_TEMPLATE_DIR/users.toml"

for MAN_FILE in $(find "$PKG_PATH/man" -type f ); do
    MAN_INSTALL_TARGET="$(echo "$MAN_FILE" | sed "s+$PKG_PATH/man+$INSTALL_MAN_DIR+")"
    $SUDO install -D -m 644 "$MAN_FILE" "$MAN_INSTALL_TARGET"
done

if [ -e "$INSTALL_CONFIG_DIR/config.toml" ]; then
    echo "info: Skipping $INSTALL_CONFIG_DIR/config.toml, it's already exists."
else
    $SUDO install -D -m 644 "$PKG_PATH/config.toml" "$INSTALL_CONFIG_DIR/config.toml"
fi

if [ -e "$INSTALL_CONFIG_DIR/users.toml" ]; then
    echo "info: Skipping $INSTALL_CONFIG_DIR/users.toml, it's already exists."
else
    $SUDO install -D -m 644 "$PKG_PATH/users.toml" "$INSTALL_CONFIG_DIR/users.toml"
fi

if [ -e "$INSTALL_CONFIG_DIR/server.key" ]; then
    echo "info: Skipping $INSTALL_CONFIG_DIR/server.key, it's already exists."
else
    if [ -z "$NUTWG_SERVER_KEY" ]; then
        echo "info: No key defined by NUTWG_SERVER_KEY, generating random key."
        NUTWG_SERVER_KEY="$(head -c 128 /dev/urandom | sha256sum -b | head -c 64)"
    fi

    echo -n "$NUTWG_SERVER_KEY" > "$TEMP_DIR/server.key"
    $SUDO install -D -m 644 "$TEMP_DIR/server.key" "$INSTALL_CONFIG_DIR/server.key"
fi

if [ $(confirm_prompt "Do you want to install service files for the selected init system ($INIT_SYSTEM)?") = 1 ]; then
    case "$INIT_SYSTEM" in
        runit)
            RUNINT_SVC_DIR="$INSTALL_PREFIX/etc/sv/nut_webgui"
            $SUDO install -D -m 755 "$PKG_PATH/service/runit/nut_webgui/run" "$RUNINT_SVC_DIR/run"
            $SUDO install -D -m 755 "$PKG_PATH/service/runit/nut_webgui/log/run" "$RUNINT_SVC_DIR/log/run"

            if [ -e "$RUNINT_SVC_DIR/conf" ]; then
                echo "info: Skipping $RUNINT_SVC_DIR/conf file, it's already exists."
            else
                $SUDO install -D -m 644 "$PKG_PATH/service/runit/nut_webgui/conf" "$RUNINT_SVC_DIR/conf"
            fi

            echo "info: nut_webgui's runit service files are installed."
            echo "Example service startup:"
            echo "    ln -s $RUNINT_SVC_DIR /var/service/"
            echo "    sv up nut_webgui"
            ;;
        openrc)
            $SUDO install -D -m 755 "$PKG_PATH/service/openrc/nut_webgui" "$INSTALL_PREFIX/etc/init.d/nut_webgui"

            if [ -e "$INSTALL_PREFIX/etc/conf.d/nut_webgui" ]; then
                echo "info: Skipping $INSTALL_PREFIX/etc/conf.d/nut_webgui file, it's already exists."
            else
                $SUDO install -D -m 644 "$PKG_PATH/service/openrc/nut_webgui.conf" "$INSTALL_PREFIX/etc/conf.d/nut_webgui"
            fi

            echo "info: nut_webgui's OpenRC service files are installed."
            echo "Example service startup:"
            echo "    rc-update add nut_webgui"
            echo "    rc-service nut_webgui start"
            ;;
        systemd)
            $SUDO install -D -m 644 "$PKG_PATH/service/systemd/nut_webgui.service" "$INSTALL_PREFIX/etc/systemd/system/nut_webgui.service"

            if [ -e "$INSTALL_CONFIG_DIR/nut_webgui.env" ]; then
                echo "info: Skipping $INSTALL_CONFIG_DIR/nut_webgui.env file, it's already exists."
            else
                $SUDO install -D -m 644 "$PKG_PATH/service/systemd/nut_webgui.env" "$INSTALL_CONFIG_DIR/nut_webgui.env"
            fi

            echo "info: nut_webgui's SystemD service files are installed."
            echo "Example service startup:"
            echo "    systemctl daemon-reload"
            echo "    systemctl enable --now nut_webgui.service"
            ;;
        *)
            echo "warn: $INIT_SYSTEM unknown init system. Skipping installation."
    esac
else
    echo "info: Skipping service installation."
fi

echo "info: nut_webgui installation completed!"
