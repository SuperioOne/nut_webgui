# Accessing localhost

An oversimplified example

```
             Host Machine
┌────────────────────────────────────┐
│                ┌─────────────────┐ │
│  NUT Service   │  Docker/Podman  │ │
│ localhost:3493 │                 │ │
│                │┌───────────────┐│ │
│                ││  nut_webgui   ││ │
│                ││   localhost   ││ │
│                │└───────────────┘│ │
│                └─────────────────┘ │
└────────────────────────────────────┘
```

When `nut_webgui` inside a container resolves 'localhost', it points to the
container's own loopback interface, not the host's. The NUT service on the host's
localhost is inaccessible from the container unless explicitly configured.

## Podman/Docker host network

Docker:
```sh
docker run --network=host -d \
  -e UPSD_ADDR=localhost \
  codeberg.org/superiorone/nut_webgui:latest
```

Podman:
```sh
podman run --network=host -d \
  -e UPSD_ADDR=localhost \
  codeberg.org/superiorone/nut_webgui:latest
```

## Podman - pasta with default host gateway (169.254.1.2)

```sh
podman run -d \
  --network=pasta:--map-gw \
  -e UPSD_ADDR=169.254.1.2 \
  -p 9000:9000 \
  codeberg.org/superiorone/nut_webgui:latest
```

## Podman - slirp4netns with default loopback IP (10.0.2.2)

> [!CAUTION]
> slirp4netns is deprecated in Podman 5. Use `Pasta` network mode instead.

```sh
podman run -d \
  --network=slirp4netns:allow_host_loopback=true \
  -e UPSD_ADDR=10.0.2.2 \
  -p 9000:9000 \
  codeberg.org/superiorone/nut_webgui:latest
```
