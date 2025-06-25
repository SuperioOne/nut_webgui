# Volumes and Args

## Attach config file directly

```bash
# Creates basic config.toml
echo 'log_level = "debug"

[upsd]
address = "10.0.0.1"
username = "admin"
password = "test"

[http_server]' > config.toml;

docker run \
  -p 9000:9000 \
  -v "$(pwd)/config.toml":"/etc/nut_webgui/config.toml" \
  ghcr.io/superioone/nut_webgui:latest
```

## Attach empty directory and use auto-generated config

`nut_webgui` automatically generates empty config file if default config file does not exist.

```bash
mkdir nut_webgui_config

docker run \
  -p 9000:9000 \
  -v "$(pwd)/nut_webgui_config":"/etc/nut_webgui/config.toml" \
  ghcr.io/superioone/nut_webgui:latest

# should be able to see/edit config.toml file
less nut_webgui_config/config.toml
```

> Config file does not auto-reload changes; you've to restart the container.

## Using CLI args

```bash
docker run \
  -p 9000:9000 \
  ghcr.io/superioone/nut_webgui:latest \
  /usr/local/bin/nut_webgui_server.sh \
  --allow-env --config-file "/etc/nut_webgui/config.toml"
```
