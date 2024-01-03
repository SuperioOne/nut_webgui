(trap 'kill 0' SIGINT; \
(pnpm run -C ./web_components dev; echo "Esbuild Stopped";) & \
(pnpm run -C ./ watch; echo "Tailwind Stopped";) & \
(cargo watch -C ./server -x "run -- --log-level debug ${UPSD_ADDR:+"--upsd-addr=$UPSD_ADDR"} ${UPSD_PORT:+"--upsd-addr=$UPSD_PORT"} ${UPSD_USR:+"--upsd-user=$UPSD_USR"} ${UPSD_PASS:+"--upsd-pass=$UPSD_PASS"}"; echo "Cargo Watch Stopped";))