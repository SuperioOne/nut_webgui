# NUT Web GUI

Lightweight web interface for [Network UPS Tools](https://networkupstools.org/).

![DetailImage](docs/src/static/views.webp)

## Quickstart

Docker/Podman:

```shell
docker run -p 9000:9000 \
  -e UPSD_ADDR=10.0.0.1 \
  -e UPSD_USER=test \
  -e UPSD_PASS=strongpass \
  ghcr.io/superioone/nut_webgui:latest
```

Container image registries:
 - **Codeberg:**
     codeberg.org/superiorone
 - **GitHub:**
     ghcr.io/superioone

## Features

- Monitors UPS variables with auto-refresh.
- Supports INSTCMD, SET VAR, and FSD calls from the GUI.
- Potato PC friendly. Small footprint in both resource usage and disk size.
- Basic JSON API.
- Supports RISC-V and older ARM devices.

## Documentation

For HTML version, see [https://nwg.smdd.dev](https://nwg.smdd.dev)

- [Introduction](./docs/src/00_introduction.md)
- [Binary installation](./docs/src/01_binary_installation.md)
- [Container images](./docs/src/02_container_images.md)
    - [Docker/Podman](./docs/src/02_01_docker_podman.md)
    - [Compose files](./docs/src/02_02_compose_files.md)
    - [Kubernetes: Basic](./docs/src/02_03_kubernetes_basic.md)
    - [Kubernetes: Endpoint slices](./docs/src/02_04_kubernetes_endpointslice.md)
- [Configuration](./docs/src/03_configuration.md)
    - [CLI args](./docs/src/03_01_cli_args.md)
    - [Environment variables](./docs/src/03_02_env_variables.md)
    - [config.toml](./docs/src/03_03_config_file.md)
- [Connecting multiple NUT servers](./docs/src/04_multiple_nut_server.md)
- [Authentication](./docs/src/05_authentication.md)
- [Path-based routing with reverse proxy](./docs/src/06_path_based_routing.md)
- [Enabling TLS on NUT connection](./docs/src/07_nut_and_tls.md)
- [Accessing host's localhost in container](./docs/src/08_accessing_localhost.md)
- [UniFi NUT](./docs/src/09_unifi_nut.md)
- [Building from source](./docs/src/10_building.md)
- [Debugging](./docs/src/11_debugging.md)
- [Data API specification](./dist/openapi3_spec.json)
- [WebSocket EventsAPI](./docs/src/13_events_api.md)
- [OpenMetrics](./docs/src/14_openmetrics.md)
- [Probes](./docs/src/15_probes.md)

## Config Templates
- [Config.toml file template](./dist/config.toml)
- [Users.toml file template](./dist/users.toml)
