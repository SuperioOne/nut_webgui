# Container images

`nut_webgui` is also distributed as a multi-arch container image and is
officially distributed on:
- ghcr.io/superioone/nut_webgui
- codeberg.org/superiorone/nut_webgui

Container images contain `musl` builds and are based on BusyBox[^busybox] base
image.

## Special directories

The application itself can be purely configured via environment variables when
the NUT deployment is simple enough. For more complex config options, use the
following directories:

- `/etc/nut_webgui`: Default directory for `config.toml`, `server.key`, and
  `users.toml`.
- `/etc/ssl/certs`: Root CA directory for TLS.

> [!IMPORTANT]
> Changes to the config files require a restart; the configs are not
> hot-reloaded.

## Environment variables

All supported environment variables are explained in the
[Configuration - 3.2. Environment variables](./03_02_env_variables.md) section.
However, there are some additional options for container-specific configuration,
and additional aliases for backward-compatibility.

### Container only options
|Name          |Default       |Value Type|Description                                                                                               |
|--------------|--------------|----------|----------------------------------------------------------------------------------------------------------|
|`UPSD_ROOT_CA`|None          |Path      |Path to the Root CA certificate for TLS. Symlinks the given path to the `/etc/ssl/certs` directory.       |
|`UID`         |1000          |1-65535   |User UID running the server. Ignored when the container already starts with a non-root user via `--user` option.|
|`GID`         |Value of `UID`|1-65535   |User GID running the server. Ignored when the container already starts with a non-root user via `--user` option.|

### Aliases

|Alias Name       |Name                           |
|-----------------|-------------------------------|
|`CONFIG_FILE`    |`NUTWG__CONFIG_FILE`           |
|`DEFAULT_THEME`  |`NUTWG__DEFAULT_THEME`         |
|`LOG_LEVEL`      |`NUTWG__LOG_LEVEL`             |
|`SERVER_KEY`     |`NUTWG__SERVER_KEY`            |
|`AUTH_USERS_FILE`|`NUTWG__AUTH__USERS_FILE`      |
|`BASE_PATH`      |`NUTWG__HTTP_SERVER__BASE_PATH`|
|`LISTEN`         |`NUTWG__HTTP_SERVER__LISTEN`   |
|`PORT`           |`NUTWG__HTTP_SERVER__PORT`     |
|`UPSD_ADDR`      |`NUTWG__UPSD__ADDRESS`         |
|`UPSD_PASS`      |`NUTWG__UPSD__PASSWORD`        |
|`POLL_FREQ`      |`NUTWG__UPSD__POLL_FREQ`       |
|`POLL_INTERVAL`  |`NUTWG__UPSD__POLL_INTERVAL`   |
|`UPSD_PORT`      |`NUTWG__UPSD__PORT`            |
|`UPSD_TLS`       |`NUTWG__UPSD__TLS_MODE`        |
|`UPSD_USER`      |`NUTWG__UPSD__USERNAME`        |

## Image tag conventions

### Upstream image tags

|Tag            |Description                                                               |
|---------------|--------------------------------------------------------------------------|
|`latest`         |Multi-arch image manifest to the most recent release.                     |
|`latest-amd64-v4`|Most recent amd64 image with micro-architecture level 4 (e.g. AVX-512).  |
|`latest-amd64-v3`|Most recent amd64 image with micro-architecture level 3 (e.g. SSE4, AVX-2).|

> [!TIP]
> **amd64/v3** and **amd64/v4** images are also available on the manifest, and
> they can be selected via the `--arch` flag instead of image tag.
> ```sh
> podman run --arch="amd64/v3" \
> -p 9000:9000 \
> codeberg.org/superiorone/nut_webgui:latest
> ```

### Version specific image tags

|Tag                       |Description                                                           |
|--------------------------|----------------------------------------------------------------------|
|0.`MAJOR`.`MINOR`         |Multi-arch image manifest with a specific version. (e.g. 0.10.2)      |
|0.`MAJOR`                 |Multi-arch image manifest with a specific `MAJOR` version. (e.g. 0.10)|
|0.`MAJOR`.`MINOR`-amd64   |amd64 image for the target version.                                   |
|0.`MAJOR`.`MINOR`-amd64-v3|amd64/v3 image for the target version.                                |
|0.`MAJOR`.`MINOR`-amd64-v4|amd64/v4 image for the target version.                                |
|0.`MAJOR`.`MINOR`-arm64   |arm64 image for the target version.                                   |
|0.`MAJOR`.`MINOR`-armv7   |arm/v7 image for the target version.                                  |
|0.`MAJOR`.`MINOR`-armv6   |arm/v6 image for the target version.                                  |
|0.`MAJOR`.`MINOR`-riscv64 |riscv64 image for the target version.                                 |

[^busybox]: [busybox.net](https://busybox.net)
