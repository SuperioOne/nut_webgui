# Accessing localhost

An over-simplified example

```
              Host Machine
┌────────────────────────────────────────┐
│                    ┌─────────────────┐ │
│   NUT Service      │  Docker/Podman  │ │
│  localhost:3493    │                 │ │
│                    │┌───────────────┐│ │
│                    ││  nut_webgui   ││ │
│                    ││   localhost   ││ │
│                    │└───────────────┘│ │
│                    └─────────────────┘ │
└────────────────────────────────────────┘
```

When `nut_webgui` inside a container uses 'localhost', it refers to the container's own loopback interface,
not the host's. NUT service on the host's localhost is inaccessible from the container unless explicitly configured.


## Podman/Docker host network

`nut_webgui` shares the same network as the host, and the UI is accessible from `localhost:9000`
without any `-p 9000:9000` port mapping.

```bash
docker run --network=host -d -e UPSD_ADDR=localhost ghcr.io/superioone/nut_webgui:latest

podman run --network=host -d -e UPSD_ADDR=localhost ghcr.io/superioone/nut_webgui:latest
```

## Podman - pasta with default host gateway (169.254.1.2)

```bash
podman run -d \
  --network=pasta:--map-gw \
  -e UPSD_ADDR=169.254.1.2 \
  -p 9000:9000 ghcr.io/superioone/nut_webgui:latest
```

## Podman - slirp4netns with default loopback IP (10.0.2.2)

> WARNING: slirp4netns is deprecated in Podman 5, prefer using `Pasta` network mode.

```bash
podman run -d \
  --network=slirp4netns:allow_host_loopback=true \
  -e UPSD_ADDR=10.0.2.2 \
  -p 9000:9000 ghcr.io/superioone/nut_webgui:latest
```
