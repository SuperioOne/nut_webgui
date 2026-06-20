#!/usr/bin/env sh
CONFIG_TEMPLATE="/usr/local/share/nut_webgui/config.toml"
DEFAULT_CONFIG_DIR="/etc/nut_webgui"
DEFAULT_ROOT_CA="/etc/ssl/certs/cert.pem"
DEFAULT_SERVER_KEY="$DEFAULT_CONFIG_DIR/server.key"
DEFAULT_CONFIG="$DEFAULT_CONFIG_DIR/config.toml"

APP_GROUP="nut_webgui"
APP_USER="nut_webgui"
UID="${UID:-"1000"}";
GID="${GID:-"$UID"}";
CURRENT_UID="$(id -u)"

# Handling aliases for backward-compability
NUTWG__AUTH__USERS_FILE="${NUTWG__AUTH__USERS_FILE:-"$AUTH_USERS_FILE"}";
NUTWG__CONFIG_FILE="${NUTWG__CONFIG_FILE:-"$CONFIG_FILE"}";
NUTWG__DEFAULT_THEME="${NUTWG__DEFAULT_THEME:-"$DEFAULT_THEME"}";
NUTWG__LOG_LEVEL="${NUTWG__LOG_LEVEL:-"$LOG_LEVEL"}";
NUTWG__SERVER_KEY="${NUTWG__SERVER_KEY:-"$SERVER_KEY"}";
NUTWG__HTTP_SERVER__BASE_PATH="${NUTWG__HTTP_SERVER__BASE_PATH:-"$BASE_PATH"}";
NUTWG__HTTP_SERVER__LISTEN="${NUTWG__HTTP_SERVER__LISTEN:-"$LISTEN"}";
NUTWG__HTTP_SERVER__PORT="${NUTWG__HTTP_SERVER__PORT:-"$PORT"}";
NUTWG__UPSD__ADDRESS="${NUTWG__UPSD__ADDRESS:-"$UPSD_ADDR"}";
NUTWG__UPSD__PASSWORD="${NUTWG__UPSD__PASSWORD:-"$UPSD_PASS"}";
NUTWG__UPSD__POLL_FREQ="${NUTWG__UPSD__POLL_FREQ:-"$POLL_FREQ"}";
NUTWG__UPSD__POLL_INTERVAL="${NUTWG__UPSD__POLL_INTERVAL:-"$POLL_INTERVAL"}";
NUTWG__UPSD__PORT="${NUTWG__UPSD__PORT:-"$UPSD_PORT"}";
NUTWG__UPSD__TLS_MODE="${NUTWG__UPSD__TLS_MODE:-"$UPSD_TLS"}";
NUTWG__UPSD__USERNAME="${NUTWG__UPSD__USERNAME:-"$UPSD_USER"}";

if [ "$CURRENT_UID" -eq "0" ]; then
    if [ "$UID" -gt "0" ]; then
        if [ -z "$(id "$APP_USER" -u 2>/dev/null)" ]; then
            if [ "$GID" -gt "0" -a "$GID" -ne "$UID" ]; then
                addgroup -g "$GID" "$APP_USER"
                adduser -D -H -G "$APP_USER" -u "$UID" "$APP_USER"
            else
                adduser -D -H -u "$UID" "$APP_USER"
            fi
        fi
    else
        APP_USER="root"
        APP_GROUP="root"
    fi

    if [ -e "$UPSD_ROOT_CA" -a ! -e "$DEFAULT_ROOT_CA" ]; then
        install -d -m 755 -o root -g "$APP_GROUP" "$(dirname $DEFAULT_ROOT_CA)"
        ln -s "$UPSD_ROOT_CA" "$DEFAULT_ROOT_CA"
    fi

    if [ -z "$NUTWG__CONFIG_FILE" ]; then
        install -d -m 775 -o root -g "$APP_GROUP" "$DEFAULT_CONFIG_DIR"

        if [ ! -e "$DEFAULT_CONFIG" ]; then
            if [ "$CURRENT_UID" -eq "0" ]; then
                install -m 664 -o root -g "$APP_GROUP" "$CONFIG_TEMPLATE" "$DEFAULT_CONFIG"
            else
                install -m 664 "$CONFIG_TEMPLATE" "$DEFAULT_CONFIG"
            fi
        fi

        NUTWG__CONFIG_FILE="$DEFAULT_CONFIG"
    fi

    if [ -z "$NUTWG__SERVER_KEY" ]; then
        if [ ! -e "$DEFAULT_SERVER_KEY" ]; then
            head -c 128 /dev/urandom | sha256sum -b | head -c 64 > "$DEFAULT_SERVER_KEY"
            if [ "$CURRENT_UID" -eq "0" ]; then
                chmod 640 "$DEFAULT_SERVER_KEY";
                chown :"$APP_GROUP" "$DEFAULT_SERVER_KEY";
            fi
        fi

        NUTWG__SERVER_KEY="$DEFAULT_SERVER_KEY";
    fi
else
    echo "WARNING: Container started with different user."
    echo "Skipping CA linking and no default config will be generated on /etc/nut_webgui."
fi

EXEC="$(cat << EOF
    export NUTWG__AUTH__ALLOW_ANONYMOUS_METRICS="$NUTWG__AUTH__ALLOW_ANONYMOUS_METRICS";
    export NUTWG__AUTH__USERS_FILE="$NUTWG__AUTH__USERS_FILE";
    export NUTWG__CONFIG_FILE="$NUTWG__CONFIG_FILE";
    export NUTWG__DEFAULT_THEME="$NUTWG__DEFAULT_THEME";
    export NUTWG__HTTP_SERVER__BASE_PATH="$NUTWG__HTTP_SERVER__BASE_PATH";
    export NUTWG__HTTP_SERVER__LISTEN="$NUTWG__HTTP_SERVER__LISTEN";
    export NUTWG__HTTP_SERVER__PORT="$NUTWG__HTTP_SERVER__PORT";
    export NUTWG__HTTP_SERVER__WORKER_COUNT="$NUTWG__HTTP_SERVER__WORKER_COUNT";
    export NUTWG__LOG_LEVEL="$NUTWG__LOG_LEVEL";
    export NUTWG__SERVER_KEY="$NUTWG__SERVER_KEY";
    export NUTWG__UPSD__ADDRESS="$NUTWG__UPSD__ADDRESS";
    export NUTWG__UPSD__MAX_CONNECTION="$NUTWG__UPSD__MAX_CONNECTION";
    export NUTWG__UPSD__NAME="$NUTWG__UPSD__NAME";
    export NUTWG__UPSD__PASSWORD="$NUTWG__UPSD__PASSWORD";
    export NUTWG__UPSD__POLL_FREQ="$NUTWG__UPSD__POLL_FREQ";
    export NUTWG__UPSD__POLL_INTERVAL="$NUTWG__UPSD__POLL_INTERVAL";
    export NUTWG__UPSD__PORT="$NUTWG__UPSD__PORT";
    export NUTWG__UPSD__TLS_MODE="$NUTWG__UPSD__TLS_MODE";
    export NUTWG__UPSD__USERNAME="$NUTWG__UPSD__USERNAME";

    exec $@
EOF
)"

if [ "$CURRENT_UID" -eq "0" ]; then
    exec su -s "/bin/sh" -c "$EXEC" "$APP_USER"
else
    exec sh -c "$EXEC"
fi
