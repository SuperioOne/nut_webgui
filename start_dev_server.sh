#!/bin/bash
set -e;

install -D "./client/static/icon.svg" "./client/dist/debug/icon.svg"

(trap 'kill 0' SIGINT; \
(pnpm run -C ./client dev-js; echo "Esbuild Stopped";) & \
(pnpm run -C ./client dev-css; echo "Tailwind Stopped";) & \
(cargo watch -C ./server -x "run -- --static-dir ../client/dist/debug --log-level debug ${UPSD_ADDR:+"--upsd-addr=$UPSD_ADDR"} ${UPSD_PORT:+"--upsd-addr=$UPSD_PORT"} ${UPSD_USR:+"--upsd-user=$UPSD_USR"} ${UPSD_PASS:+"--upsd-pass=$UPSD_PASS"}"; echo "Cargo Watch Stopped";))
