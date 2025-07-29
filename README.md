# NUT Web GUI

Light weight web interface for [Network UPS Tools](https://networkupstools.org/).

<div style="width: 100%; display: flex; justify-content: space-between; flex-direction: row; gap: 1rem;">

![DetailImage](docs/images/details.webp)

![ListImage](docs/images/home.webp)

</div>

## Quickstart

```shell
docker run -p 9000:9000 \
  -e UPSD_ADDR=10.0.0.1 \
  -e UPSD_USER=test \
  -e UPSD_PASS=strongpass \
  ghcr.io/superioone/nut_webgui:latest
```

## Features

- Monitors UPS variables with auto refresh.
- Supports INSTCMD, SET VAR, and FSD calls from GUI.
- 🥔 Potato PC friendly. Small footprint on both resource usage and disk size.
- Basic JSON API.
- Supports RISC-V and older ARM devices.

> In order to run `INSTCMD` and `FSD`, make sure the configured user has proper privileges given at `upsd.users`. See
> man([upsd.users](https://networkupstools.org/docs/man/upsd.users.html)).

## Examples

- [Volumes and args](docs/examples/01_volumes_and_args.md)
- [Docker/Podman compose](docs/examples/02_compose.md)
- [Kubernetes - Basic](docs/examples/03_kubernetes_basic.md)
- [Kubernetes - EndpointSlice](docs/examples/04_kubernetes_endpointslice.md)
- [Custom base path for reverse proxy](docs/examples/05_reverse_proxy_base_path.md)
- [Accessing localhost](docs/examples/06_accessing_localhost.md)
- [Using NUT with TLS](docs/examples/07_nut_and_tls.md)

## CPU architecture support

| Arch         | Test Hardware           | Notes                                                                                         |
|--------------|-------------------------|-----------------------------------------------------------------------------------------------|
| amd64        | AM4 CPU                 | Works across all amd64 platforms.                                                             |
| amd64-v3     | AM4 CPU                 | Snake oil level optimizations with AVX. It mostly improves response compression, and TLS.     |
| amd64-v4     | Intel® SDE              | Snake oil level optimizations with AVX-512. It mostly improves response compression, and TLS. |
| arm64        | Raspberry Pi 4 Model B  |                                                                                               |
| armv7        | Qemu emulation          | Uses software floating-point.                                                                 |
| armv6        | Qemu emulation          | Uses software floating-point.                                                                 |
| riscv64      | Qemu emulation          |                                                                                               |

> amd64 v3 and v4 variants require certain CPU feature flags to run. If you are a crackhead min-max enjoyer (like me), you can use 
> `nut_webgui:latest-amd64-v3` and `nut_webgui:latest-amd64-v4` images.
>
> See [x86-64 Microarchitecture levels](https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels) for more details.


## Configuration

nut_webgui can be configured via args, environment variables, or config file. All configuration options are merged into single unified config based on their priority.

### CLI arguments

CLI arguments hold the highest priority in configuration settings. You can override existing configurations by using them.

* `--allow-env`: Allows application to load configuration from environment variables.
* `--base-path`: Overrides HTTP server base path. Default is `/`.
* `--config-file`: config.toml path.
* `--default-theme`: Web UI default theme.
* `--listen`: Listen address for the HTTP server. Default is `0.0.0.0`.
* `--log-level`: Log level for the HTTP server. Default is `info`.
* `--poll-freq`: UPS [pollfreq](https://networkupstools.org/docs/man/ups.conf.html#_global_directives) in seconds. Default is `30`.
* `--poll-interval`: UPS [pollinterval](https://networkupstools.org/docs/man/ups.conf.html#_global_directives) in seconds. Default is `2`.
* `--port`: Port used by the HTTP server. Default is `9000`.
* `--upsd-addr`: UPS daemon address. Default is `localhost`.
* `--upsd-max-connection`: Allowed maximum connection for UPSD client. Default is `4`.
* `--upsd-pass`: UPS daemon password.
* `--upsd-port`: UPS daemon port. Default is `3493`.
* `--upsd-user`: UPS daemon username.
* `--upsd-tls-mode`: Configures TLS communication between UPSD and client. Default is `disable`.

### Container image environment variables

Environment variables have the second-highest priority in configuration settings. They can also accept paths as values. When an environment variable 
specifies a file path, the system automatically reads content of that file as a value.

| Names                                         | Default                        | Description                                                        |
|-----------------------------------------------|--------------------------------|--------------------------------------------------------------------|
| `CONFIG_FILE`, `NUTWG__CONFIG_FILE`           | `/etc/nut_webgui/config.toml`  | custom config.toml file path.                                      |
| `LOG_LEVEL`, `NUTWG__LOG_LEVEL`               | `info`                         | Log level.                                                         |
| `DEFAULT_THEME`, `NUTWG__DEFAULT_THEME`       | None                           | Web UI default theme.                                              |
| `BASE_PATH`, `NUTWG__HTTP_SERVER__BASE_PATH`  | `/`                            | Overrides HTTP server base path.                                   |
| `LISTEN`, `NUTWG__HTTP_SERVER__LISTEN`        | `0.0.0.0`                      | Works across all amd64 platforms.                                  |
| `PORT`, `NUTWG__HTTP_SERVER__PORT`            | `9000`                         | Works across all amd64 platforms.                                  |
| `NUTWG__UPSD__MAX_CONNECTION`                 | `4`                            | Allowed maximum connection for UPSD client.                        |
| `POLL_FREQ`, `NUTWG__UPSD__POLL_FREQ`         | `30`                           | Non-critical ups variables update frequency in seconds.            |
| `POLL_INTERVAL`, `NUTWG__UPSD__POLL_INTERVAL` | `2`                            | Critical ups variables (`ups.status`) update interval in seconds.  |
| `UPSD_ADDR`, `NUTWG__UPSD__ADDRESS`           | `localhost`                    | UPS daemon address.                                                |
| `UPSD_PASS`, `NUTWG__UPSD__PASSWORD`          | None                           | UPS daemon password.                                               |
| `UPSD_PORT`, `NUTWG__UPSD__PORT`              | `3493`                         | UPS daemon port.                                                   |
| `UPSD_USER`, `NUTWG__UPSD__USERNAME`          | None                           | UPS daemon username.                                               |
| `UPSD_TLS`, `NUTWG__UPSD__TLS_MODE`           | `disable`                      | Configures TLS communication between UPSD and client.              |
| `UPSD_ROOT_CA`                                | None                           | Path to the Root CA certificate for TLS.                           |

### TOML config

Config.toml has the least priority, but it's recommended to use the config file as a baseline configuration and use environment variables or command-line arguments 
to override settings.

```toml
log_level = "info"
default_theme = "tokyo-night"

[http_server]
base_path = "/"
listen = "0.0.0.0"
port = 9000

[upsd]
username = "admin"
password = "test"
address = "localhost"
port = 3493
max_connection = 4
poll_freq = 30
poll_interval = 2
```

For more detailed config template see [./containers/config.toml](./containers/config.toml).

> Log level options: `info`, `warn`, `error`, `debug`, `trace`

## JSON data API

A simple JSON-based API is available for integration and automation purposes.

OpenAPI 3.0.0 specification files: [json](docs/api_specs/openapi3_spec.json) | [yaml](docs/api_specs/openapi3_spec.yaml)

## Probes

nut_webgui has basic probe endpoints to check server health and readiness:
- `/probes/health`
- `/probes/readiness`

## Building from source and debugging

[Building and Debugging](./docs/building_debugging.md)

[Custom Build Pipeline](./docs/custom_build_pipeline.md)
