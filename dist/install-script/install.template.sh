#!/usr/bin/env sh
set -e;

# Inspired from the first version of the k3s install.sh
# attribution: @ibuildthecloud

NUTWG_VERSION="__PLACEHOLDER_NUTWG_VERSION";
NUTWG_RELEASE_URL="https://github.com/SuperioOne/nut_webgui/releases/download/v${NUTWG_VERSION}";
NUTWG_INSTALL_PATH="/usr/local/bin/nut_webgui"
NUTWG_CONFIG_DIR="/etc/nut_webgui"

detect_init_system() {
    if test -e "/sbin/init"; then
        __INIT_SYSTEM_PATH="$(realpath "/sbin/init")"
        __INIT_SYSTEM="$(basename "$__INIT_SYSTEM_PATH")"

        echo "$__INIT_SYSTEM"

        #NOTE: More options can be added such as openrc, /etc/init.d
        case "$__INIT_SYSTEM" in
            systemd)
                NUTWG_INIT_SYSTEM="systemd"
                ;;
            *)
                ;;
        esac
    fi
}

detect_libc() {
    if getconf "GNU_LIBC_VERSION" 2>&1 > /dev/null; then
        NUT_LIBC_TYPE="gnu"
    else
        NUT_LIBC_TYPE="musl"
    fi
}

detect_target() {
    if test -z "$ARCH"; then
        NUTWG_ARCH="$(uname -m)"
    else
        NUTWG_ARCH="$ARCH"
    fi

    if test "$NUTWG_ARCH" = "amd64"; then
        NUTWG_ARCH="x86_64"
    elif test "$NUTWG_ARCH" = "arm64"; then
        NUTWG_ARCH="aarch64"
    fi

    case "$NUTWG_ARCH" in
        x86_64)
            if test "$NUT_LIBC_TYPE" = "musl"; then
                NUTWG_TARGET="x86-64-musl"
            else
                NUTWG_TARGET="x86-64-gnu"
            fi
            ;;
        aarch64)
            if test "$NUT_LIBC_TYPE" = "musl"; then
                NUTWG_TARGET="aarch64-musl"
            else
                NUTWG_TARGET="aarch64-gnu"
            fi
            ;;
        riscv64)
            NUTWG_TARGET="riscv64gc-gnu"
            ;;
        armv7)
            NUTWG_TARGET="armv7-musleabi"
            ;;
        armv6)
            NUTWG_TARGET="armv6-musleabi"
            ;;
        *)
            echo "err: unsupported CPU architecture: $ARCH"
            echo "If you think script detection is incorrect, try setting NUTWG_TARGET env variable with the one of the following options:"
            echo "  aarch64-gnu"
            echo "  aarch64-musl"
            echo "  armv6-musleabi"
            echo "  armv7-musleabi"
            echo "  riscv64gc-gnu"
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
    if which curl &>/dev/null; then
        DOWNLOAD_CLIENT="curl"
    elif which wget &>/dev/null; then
        DOWNLOAD_CLIENT="wget"
    else
        echo "err: install script requires curl or wget to download binaries".
        exit 1;
    fi
}

download() {
    case "$DOWNLOAD_CLIENT" in
        curl)
            curl -fL --output-dir "$2" -O "$1"
            ;;
        wget)
            wget --backups=3 -q --show-progress --directory-prefix="$2" "$1"
            ;;
        *)
            echo "No download client is set"
            exit 1
            ;;
    esac
}

default_systemd_service() {
    cat <<EOF
[Unit]
Description=nut_webgui - Simple NUT Web interface
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=${NUTWG_INSTALL_PATH} --config-file ${NUTWG_CONFIG_DIR}/config.toml --allow-env
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
}

SUDO=
if test "$(id -u)" -ne 0; then
    SUDO=sudo
fi

if test -z "$NUTWG_TARGET"; then
    detect_target
fi

detect_download_client
detect_init_system

echo "detected target: $NUTWG_TARGET"
echo "detected init system: ${NUTWG_INIT_SYSTEM:-"N/A"}"

NUTWG_PACKAGE="nut_webgui_${NUTWG_VERSION}_${NUTWG_TARGET}.tar.gz"
NUTWG_PACKAGE_SHA="nut_webgui_${NUTWG_VERSION}_${NUTWG_TARGET}.tar.gz.sha256"
NUTWG_BIN_URL="$NUTWG_RELEASE_URL/$NUTWG_PACKAGE"
NUTWG_SHA_URL="$NUTWG_RELEASE_URL/$NUTWG_PACKAGE_SHA"

echo "downloading: $NUTWG_SHA_URL";
download "$NUTWG_SHA_URL" "/tmp";

echo "downloading: $NUTWG_BIN_URL";
download "$NUTWG_BIN_URL" "/tmp";

if cat "/tmp/$NUTWG_PACKAGE_SHA" | sed "s/\\s.*\$/ \/tmp\/$NUTWG_PACKAGE/" | sha256sum --status -c; then
    tar -xf "/tmp/$NUTWG_PACKAGE" -C /tmp
else
    echo "err: sha256sum check failed for the downloaded $NUTWG_PACKAGE"
    exit 1
fi

$SUDO install -m=751 "/tmp/${NUTWG_TARGET}/nut_webgui" "${NUTWG_INSTALL_PATH}"
echo "nut_webgui executable installed to ${NUTWG_INSTALL_PATH}"

if test ! -d "$NUTWG_CONFIG_DIR"; then
    __TMP_CONF="$(mktemp -t config.toml.XXX)"
    echo 'version = "1"' > "$__TMP_CONF"
    $SUDO install -m=744 -D "$__TMP_CONF" "${NUTWG_CONFIG_DIR}/config.toml";
    echo "empty configuration file created at: ${NUTWG_CONFIG_DIR}/config.toml"
else
    echo "$NUTWG_CONFIG_DIR already exists, skipping any default config.toml creation."
fi

case "$NUTWG_INIT_SYSTEM" in
    systemd)
        if test -d "/etc/systemd/system/" ; then
            if test ! -e "/etc/systemd/system/nut_webgui.service";then
                __TMP_SVC="$(mktemp -t nut_webgui.service.XXX)"
                default_systemd_service > "$__TMP_SVC"
                $SUDO install -m=755 -D "$__TMP_SVC" "/etc/systemd/system/nut_webgui.service"

                echo "optional systemd unit for nut_webgui is created at /etc/systemd/system/nut_webgui.service"
                echo "to enable systemd unit use:"
                echo "  systemctl daemon-reload"
                echo "  systemctl enable --now nut_webgui.service"
            else
                echo "systemd unit is already exists, skipping creating a new service"
            fi
        fi
        ;;
    *)
        echo "no init system has been found, skipping auto-start configuration"
        ;;
esac
