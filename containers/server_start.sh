#!/usr/bin/env sh
set -e
BIN_LOCATION="/usr/local/bin/nut_webgui"
DEFAULT_CONFIG="/usr/local/share/nut_webgui/config.toml"
SYSTEM_CONFIG_DIR="/etc/nut_webgui"
ROOT_CA_TARGET="/usr/local/ssl/cert.pem"

function generate_rand_key {
    cat /dev/urandom | head -c 128 | sha256sum -b -z | head -c 64 -z
}

# Aliases
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

if test ! -e "$SYSTEM_CONFIG_DIR/config.toml"; then
    install -D -m 664 "$DEFAULT_CONFIG" "$SYSTEM_CONFIG_DIR"
fi

if test -z "$NUTWG__SERVER_KEY" -a ! -e "$SYSTEM_CONFIG_DIR/server.key"; then
    generate_rand_key > "$SYSTEM_CONFIG_DIR/server.key";
    export NUTWG__SERVER_KEY="$SYSTEM_CONFIG_DIR/server.key";
fi

if test -e "$UPSD_ROOT_CA" -a ! -e "$ROOT_CA_TARGET"; then
    ln -s "$UPSD_ROOT_CA" "$ROOT_CA_TARGET"
fi

# If it's still not set, fallback to default config file
export NUTWG__CONFIG_FILE="${NUTWG__CONFIG_FILE:-"$SYSTEM_CONFIG_DIR/config.toml"}";

if test $# -gt 0; then
    exec "$BIN_LOCATION" $@;
else
    exec "$BIN_LOCATION" --allow-env;
fi
