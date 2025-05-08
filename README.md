# NUT Web GUI

Light weight web interface for [Network UPS Tools](https://networkupstools.org/).

<div style="width: 100%; display: flex; justify-content: space-between; flex-direction: row; gap: 1rem;">

![DetailImage](docs/images/details.webp)

![ListImage](docs/images/home.webp)

</div>

## Quickstart

```shell
docker run -e UPSD_ADDR=10.0.0.1 -e UPSD_USER=test -e UPSD_PASS=strongpass -p 9000:9000 ghcr.io/superioone/nut_webgui:latest
```

## Features

- Monitors UPS variables with auto refresh.
- Supports INSTCMD calls from GUI.
- 🥔 Potato PC friendly. Small footprint on both resource usage and disk size.
- Basic JSON API.
- Supports RISC-V and older ARM devices.

> In order to run `INSTCMD`, make sure the configured user has proper instcmds granted at `upsd.users`. See
> man([upsd.users](https://networkupstools.org/docs/man/upsd.users.html)).

## Examples

- [Kubernetes with EndpointSlice](docs/kubernetes_example.md)
- [Docker compose](docs/docker_compose.md)

## CPU architecture support

| Arch         | Test Hardware           | Notes                                                                                    |
|--------------|-------------------------|------------------------------------------------------------------------------------------|
| amd64        | AM4 CPU                 | Works across all amd64 platforms.                                                        |
| amd64-v3     | AM4 CPU                 | Snake oil level optimizations with AVX. It mostly reduces response compression overhead. |
| amd64-v4     | Intel® SDE              | Snake oil level optimizations with AVX-512.                                              |
| arm64        | Raspberry Pi 4 Model B  |                                                                                          |
| armv7        | Qemu emulation          |                                                                                          |
| armv6        | Qemu emulation          |                                                                                          |
| riscv64      | Qemu emulation          |                                                                                          |

> amd64 v3 and v4 variants require certain CPU feature flags to run. If you are a crackhead min-max enjoyer (like me), you can use `nut_webgui:latest-amd64-v3` and `nut_webgui:latest-amd64-v4` images.
> See [x86-64 Microarchitecture levels](https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels) for more details.


## Command-Line arguments

* `--base-path`: Overrides HTTP server base path. Default is `/`.
* `--listen`: Listen address for the HTTP server. Default is `0.0.0.0`.
* `--log-level`: Log level for the HTTP server. Default is `info`.
* `--poll-freq`: UPS[pollfreq](https://networkupstools.org/docs/man/ups.conf.html#_global_directives) in seconds. Default is `30`.
* `--poll-interval`: UPS [pollinterval](https://networkupstools.org/docs/man/ups.conf.html#_global_directives) in seconds. Default is `2`.
* `--port`: Port used by the HTTP server. Default is `9000`.
* `--static-dir`: Directory path for static files. Default is `./static`.
* `--upsd-addr`: UPS daemon address. Default is `localhost`.
* `--upsd-pass`: UPS daemon password.
* `--upsd-port`: UPS daemon port. Default is `3493`.
* `--upsd-user`: UPS daemon username.

## Container image environment variables

* `BASE_PATH`: Overrides HTTP server base path. Default is `/`.
* `LISTEN`: Listen address for the HTTP server. Default is `0.0.0.0`.
* `LOG_LEVEL`: Log level. Default is `info`.
* `POLL_FREQ`: UPS [pollfreq](https://networkupstools.org/docs/man/ups.conf.html#_global_directives) in seconds. Default is `30`.
* `POLL_INTERVAL`: UPS [pollinterval](https://networkupstools.org/docs/man/ups.conf.html#_global_directives) in seconds. Default is `2`.
* `PORT`: Port used by the HTTP server. Default is `9000`.
* `UPSD_ADDR`: UPS daemon address. Default is `localhost`.
* `UPSD_PASS`: UPS daemon password.
* `UPSD_PORT`: UPS daemon port. Default is `3493`.
* `UPSD_USER`: UPS daemon username.

> Log level options: `info`, `warn`, `error`, `debug`, `trace`

> `BASE_PATH` rules:
>
> - Allows multiple path segments. (eg. `segment1/segment2/...`)
> - Whitespaces and trailing forward slash `/` characters will be trimmed. For example, `  base_path   ` and `/base_path/` are treated as the same.
> - Percent encoded characters are not supported.
> - Unlike RFC3986 path definition, `:` and `'` characters are not allowed. However, you can still use some questionable base paths such as `(-_-)/(@_@)/($_$)/nut-xyz.monitor/`

## JSON data API

A simple JSON-based API is available for integration and automation purposes.

OpenAPI 3.1.0 specification files: [json minimized](./docs/openapi3_spec.json) | [yaml](./docs/openapi3_spec.yaml)

## Probes

nut_webgui has basic probe endpoints to check server health and readiness:
- `/probes/health`
- `/probes/readiness`

## Building from source and debugging

See [Building and Debugging.](./docs/building_debugging.md)
