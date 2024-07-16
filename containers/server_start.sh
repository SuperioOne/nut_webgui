#!/usr/bin/env sh
set -e
BIN_LOCATION=/opt/nut_webgui/nut_webgui
STATIC_LOCATION=/opt/nut_webgui/static

exec "$BIN_LOCATION" --static-dir "$STATIC_LOCATION" \
${LISTEN:+"--listen=$LISTEN"} \
${PORT:+"--port=$PORT"} \
${POLL_FREQ:+"--poll-freq=$POLL_FREQ"} \
${UPSD_PORT:+"--upsd-port=$UPSD_PORT"} \
${UPSD_ADDR:+"--upsd-addr=$UPSD_ADDR"} \
${UPSD_USER:+"--upsd-user=$UPSD_USER"} \
${UPSD_PASS:+"--upsd-pass=$UPSD_PASS"} \
${LOG_LEVEL:+"--log-level=$LOG_LEVEL"}
