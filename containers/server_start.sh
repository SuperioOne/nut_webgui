#!/usr/bin/env sh
set -e
BIN_LOCATION=/opt/nut_webgui/nut_webgui
STATIC_LOCATION=/opt/nut_webgui/static

exec "$BIN_LOCATION" --static-dir "$STATIC_LOCATION" \
    ${LISTEN:+"--listen"} ${LISTEN:+"${LISTEN}"} \
    ${PORT:+"--port"} ${PORT:+"${PORT}"} \
    ${POLL_FREQ:+"--poll-freq"} ${POLL_FREQ:+"${POLL_FREQ}"} \
    ${POLL_INTERVAL:+"--poll-interval"} ${POLL_INTERVAL:+"${POLL_INTERVAL}"} \
    ${UPSD_PORT:+"--upsd-port"} ${UPSD_PORT:+"${UPSD_PORT}"} \
    ${UPSD_ADDR:+"--upsd-addr"} ${UPSD_ADDR:+"${UPSD_ADDR}"} \
    ${UPSD_USER:+"--upsd-user"} ${UPSD_USER:+"${UPSD_USER}"} \
    ${UPSD_PASS:+"--upsd-pass"} ${UPSD_PASS:+"${UPSD_PASS}"} \
    ${LOG_LEVEL:+"--log-level"} ${LOG_LEVEL:+"${LOG_LEVEL}"} \
    ${BASE_PATH:+"--base-path"} ${BASE_PATH:+"${BASE_PATH}"}
