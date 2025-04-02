#!/bin/bash
set -e;

install -D "./nut_webgui_client/static/icon.svg" "./nut_webgui/dist/debug/icon.svg"

# Makes sures node_modules folder is initializes.
pnpm install -C ./nut_webgui_client/

trap 'echo "Shutting down dev server"; exit;' INT KILL ABRT;

(pnpm run -C ./nut_webgui_client dev-js; echo "Esbuild Stopped";) \
    & (pnpm run -C ./nut_webgui_client dev-css; echo "Tailwind Stopped";) \
    & (cargo watch -C ./nut_webgui -x "run -- --static-dir ../nut_webgui_client/dist/debug --log-level debug ${UPSD_ADDR:+"--upsd-addr=$UPSD_ADDR"} ${UPSD_PORT:+"--upsd-addr=$UPSD_PORT"} ${UPSD_USR:+"--upsd-user=$UPSD_USR"} ${UPSD_PASS:+"--upsd-pass=$UPSD_PASS"}"; echo "Cargo Watch Stopped";)
