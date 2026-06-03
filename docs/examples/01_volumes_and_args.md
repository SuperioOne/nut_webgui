# Configuration: Volumes and Arguments

## Mounting a configuration file

Create a `config.toml` on your host and mount it into the container:

```bash
echo 'version = "1"

[upsd.default]
address = "10.0.0.1"
username = "admin"
password = "test"

' > config.toml;
```

*Start nut_webgui with the `config.toml` file*
```bash
docker run \
  -p 9000:9000 \
  -v "$(pwd)/config.toml":"/etc/nut_webgui/config.toml" \
  ghcr.io/superioone/nut_webgui:latest
```

## Using an auto-generated configuration

If no config exists at `/etc/nut_webgui/config.toml`, `nut_webgui` generates one
automatically. You can mount an empty directory to `/etc/nut_webgui` to persist
generated file.

```bash
mkdir app_config

docker run \
  -p 9000:9000 \
  -v "$(pwd)/app_config":"/etc/nut_webgui" \
  ghcr.io/superioone/nut_webgui:latest
```

After container starts, generated `config.toml` can be read/edited on the host
machine.

```bash
cat app_config/config.toml
```

> **Note:** Changes require a container restart; the config is not hot-reloaded.

## Using CLI arguments

You can also override settings by passing arguments directly to the server
command.

```bash
docker run \
  -p 9000:9000 \
  ghcr.io/superioone/nut_webgui:latest \
  /usr/local/bin/nut_webgui_server.sh \
  --allow-env --config-file "/etc/nut_webgui/config.toml"
```
