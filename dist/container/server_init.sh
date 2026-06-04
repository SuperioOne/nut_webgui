#!/usr/bin/env sh
set -e
BIN_LOCATION="/usr/local/bin/nut_webgui"
CONFIG_TEMPLATE="/usr/local/share/nut_webgui/config.toml"
SYSTEM_CONFIG_DIR="/etc/nut_webgui"
ROOT_CA_TARGET="/etc/ssl/certs/cert.pem"

DEFAULT_SERVER_KEY="$SYSTEM_CONFIG_DIR/server.key"
DEFAULT_CONFIG="$SYSTEM_CONFIG_DIR/config.toml"

APP_GROUP="nut_webgui"
APP_USER="nut_webgui"
UID="${UID:-"1000"}";
GID="${GID:-"$UID"}";

if test "$UID" -ne "1000" -a "$UID" -gt "0"; then
    APP_USER="appuser"

    if test "$GID" -eq "1000"; then
        adduser -D -H -G "$APP_GROUP" -u "$UID" "$APP_USER"
    elif test "$GID" -gt "0" -a "$GID" -ne "$UID"; then
        addgroup -g "$GID" "$APP_USER"
        adduser -D -H -G "$APP_USER" -u "$UID" "$APP_USER"
    else
        adduser -D -H -u "$UID" "$APP_USER"
    fi

    addgroup "$APP_USER" "$APP_GROUP"
elif test "$UID" -eq "0"; then
    APP_USER="root"
else
    APP_USER="nut_webgui"

    if test "$GID" -gt "0" -a "$GID" -ne "$UID"; then
        addgroup -g "$GID" "$APP_GROUP"
        adduser "$APP_USER" "$APP_GROUP"
    fi
fi

if test -e "$UPSD_ROOT_CA" -a ! -e "$ROOT_CA_TARGET"; then
    ln -s "$UPSD_ROOT_CA" "$ROOT_CA_TARGET"
fi

EXEC="$(cat << EOF
    if test -z "$CONFIG_FILE" -a ! -e "$DEFAULT_CONFIG"; then
        install -m 664 "$CONFIG_TEMPLATE" "$DEFAULT_CONFIG"
    fi

    export NUTWG__AUTH__USERS_FILE="${NUTWG__AUTH__USERS_FILE:-"$AUTH_USERS_FILE"}";
    export NUTWG__CONFIG_FILE="${NUTWG__CONFIG_FILE:-"$CONFIG_FILE"}";
    export NUTWG__DEFAULT_THEME="${NUTWG__DEFAULT_THEME:-"$DEFAULT_THEME"}";
    export NUTWG__HTTP_SERVER__BASE_PATH="${NUTWG__HTTP_SERVER__BASE_PATH:-"$BASE_PATH"}";
    export NUTWG__HTTP_SERVER__LISTEN="${NUTWG__HTTP_SERVER__LISTEN:-"$LISTEN"}";
    export NUTWG__HTTP_SERVER__PORT="${NUTWG__HTTP_SERVER__PORT:-"$PORT"}";
    export NUTWG__LOG_LEVEL="${NUTWG__LOG_LEVEL:-"$LOG_LEVEL"}";
    export NUTWG__SERVER_KEY="${NUTWG__SERVER_KEY:-"$SERVER_KEY"}";
    export NUTWG__UPSD__ADDRESS="${NUTWG__UPSD__ADDRESS:-"$UPSD_ADDR"}";
    export NUTWG__UPSD__PASSWORD="${NUTWG__UPSD__PASSWORD:-"$UPSD_PASS"}";
    export NUTWG__UPSD__POLL_FREQ="${NUTWG__UPSD__POLL_FREQ:-"$POLL_FREQ"}";
    export NUTWG__UPSD__POLL_INTERVAL="${NUTWG__UPSD__POLL_INTERVAL:-"$POLL_INTERVAL"}";
    export NUTWG__UPSD__PORT="${NUTWG__UPSD__PORT:-"$UPSD_PORT"}";
    export NUTWG__UPSD__TLS_MODE="${NUTWG__UPSD__TLS_MODE:-"$UPSD_TLS"}";
    export NUTWG__UPSD__USERNAME="${NUTWG__UPSD__USERNAME:-"$UPSD_USER"}";

    if test -z "$NUTWG__SERVER_KEY" -a ! -e "$DEFAULT_SERVER_KEY"; then
        cat /dev/urandom | head -c 128 | sha256sum -b | head -c 64 > "$DEFAULT_SERVER_KEY"
        export NUTWG__SERVER_KEY="$DEFAULT_SERVER_KEY";
    fi

    export NUTWG__CONFIG_FILE="${NUTWG__CONFIG_FILE:-"$DEFAULT_CONFIG"}";

    exec $@
EOF
)"

su -s "/bin/sh" -c "$EXEC" "$APP_USER"
