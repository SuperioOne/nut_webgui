# NUT Web GUI

[![version:0.1.1](https://img.shields.io/badge/version-0.2.0-red)](https://github.com/SuperioOne/nut_webgui/releases/tag/v0.2.0)
[![version:0.1.1](https://img.shields.io/badge/linux/amd64-0.2.0-green)](https://github.com/SuperioOne/nut_webgui/pkgs/container/nut_webgui)
[![version:0.1.1](https://img.shields.io/badge/linux/arm64-0.2.0-green)](https://github.com/SuperioOne/nut_webgui/pkgs/container/nut_webgui)

Web-based simple interface for [Network UPS Tools](https://networkupstools.org/).

**Quickstart:**

```shell
docker run --rm -e UPSD_ADDR=10.0.0.1 -e UPSD_USER=test -e UPSD_PASS=strongpass -p 9000:9000 ghcr.io/superioone/nut_webgui:latest   
```

## Features

- Monitoring UPS variables with auto refresh.
- List supported commands by UPS and allows INSTCMD calls from GUI.
- Lightweight and small.

> In order to run `INSTCMD`, make sure the configured user has proper instcmds granted at `upsd.users`. See
> man([upsd.users](https://networkupstools.org/docs/man/upsd.users.html)).

![DetailImage](docs/images/details.webp)

![ListImage](docs/images/list.webp)

![INSTCMDImage](docs/images/inst_cmd.webp)

## Command-Line Arguments

The following command-line arguments can be used to configure the application:

* `--poll-freq`: Specify the poll frequency in seconds. Default is `10`.
* `--upsd-addr`: Specify the UPS daemon address. Default is `localhost`.
* `--upsd-port`: Specify the UPS daemon port. Default is `3493`.
* `--upsd-user`: Specify the UPS daemon username.
* `--upsd-pass`: Specify the UPS daemon password.
* `--listen`: Specify the listen address for the HTTP server. Default is `0.0.0.0`.
* `--port`: Specify the port used by the HTTP server. Default is `9000`.
* `--log-level`: Specify the log level for the HTTP server. Default is `info`.
* `--static-dir`: Specify the location of static css and js files served by the HTTP server. Default is `./static`.

## Container Image Environment Variables

The following environment variables can be used to configure the application:

* `POLL_FREQ`: Specify the poll frequency in seconds. Default is `10`.
* `UPSD_ADDR`: Specify the UPS daemon address. Default is `localhost`.
* `UPSD_PORT`: Specify the UPS daemon port. Default is `3493`.
* `UPSD_USER`: Specify the UPS daemon username.
* `UPSD_PASS`: Specify the UPS daemon password.
* `LISTEN`: Specify the listen address for the HTTP server. Default is `0.0.0.0`.
* `PORT`: Specify the port used by the HTTP server. Default is `9000`.
* `LOG_LEVEL`: Specify the log level for the HTTP server. Default is `info`.

## JSON Data API

A simple JSON-based API is available for integration and automation purposes. For usage details
see [JSON data API page.](./docs/json_api.md)

## Probes

The server offers a basic health probe at the `/probes/health` endpoint. This can be used as liveness or readiness
checks in your systems.

## Examples

- [Kubernetes with EndpointSlice](docs/kubernetes_example.md)
- [Docker compose](docs/docker_compose.md)
