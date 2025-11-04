# Configuration: Volumes and Arguments

## 1. Mounting a Configuration File

You can create a `config.toml` file on your host machine and mount it directly into the container.

*Create a basic `config.toml`*
```bash
echo 'version = "1"

[upsd.default]
address = "10.0.0.1"
username = "admin"
password = "test"

' > config.toml;
```

*Run nut_webgui with the `config.toml` file*
```bash
docker run \
  -p 9000:9000 \
  -v "$(pwd)/config.toml":"/etc/nut_webgui/config.toml" \
  ghcr.io/superioone/nut_webgui:latest
```

## 2. Using an Auto-Generated Configuration

If a configuration file is not found at the default location (`/etc/nut_webgui/config.toml`), `nut_webgui` will automatically generate a new one. You can mount a directory to this location to persist the generated file and edit it on the host.

*Create an empty directory and mount it to nut_webgui's config directory*
```bash
mkdir app_config

docker run \
  -p 9000:9000 \
  -v "$(pwd)/app_config":"/etc/nut_webgui" \
  ghcr.io/superioone/nut_webgui:latest
```

*After container starts, generated `config.toml` can be read/edited on the host machine*
```bash
cat app_config/config.toml
```

> **Note:** The container must be restarted for any changes to the configuration file to take effect. The file is not reloaded automatically.

## 3. Using CLI Arguments

You can also override settings by passing arguments directly to the server command.

```bash
docker run \
  -p 9000:9000 \
  ghcr.io/superioone/nut_webgui:latest \
  /usr/local/bin/nut_webgui_server.sh \
  --allow-env --config-file "/etc/nut_webgui/config.toml"
```
