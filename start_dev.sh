#!/bin/bash
set -e;

pnpm install -C ./nut_webgui_client/

trap 'echo "Shutting down dev server"; exit;' INT KILL ABRT;

cargo watch \
    -C ./nut_webgui \
    -x "run -- --allow-env"; 

echo "Cargo Watch Stopped";
