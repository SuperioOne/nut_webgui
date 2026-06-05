# Configuration: Volumes and Arguments

`nut_webgui` has two main configuration files you may want to persist:

- `/etc/nut_webgui/config.toml`: primary config file.
- `/etc/nut_webgui/server.key`: server signing key.

> **NOTE:** Changes to the files require a container restart; the configs are
not hot-reloaded.

> **IMPORTANT:** When the container user is overridden using the `--user` option,
> the container will not automatically generate the default configuration file
> or server key. You are responsible for mounting the configuration files with
> the correct permissions and setting up any required environment variables.

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

Start nut_webgui with the `config.toml` file

```bash
docker run \
  -p 9000:9000 \
  -v "$(pwd)/config.toml":"/etc/nut_webgui/config.toml" \
  ghcr.io/superioone/nut_webgui:latest
```

## Mounting directory as volume

You can mount an empty directory to `/etc/nut_webgui` to persist configurations.

If no config exists at `/etc/nut_webgui/config.toml`, `nut_webgui` generates one
automatically.

```bash
mkdir app_config

echo 'version = "1"

[upsd.default]
address = "10.0.0.1"
username = "admin"
password = "test"

' > ./app_config/config.toml;

docker run \
  -p 9000:9000 \
  -v "$(pwd)/app_config":"/etc/nut_webgui" \
  ghcr.io/superioone/nut_webgui:latest
```

## Using CLI arguments

You can also override settings by passing arguments directly to the server
command.

```bash
docker run \
  -p 9000:9000 \
  ghcr.io/superioone/nut_webgui:latest \
  nut_webgui --allow-env --log-level "debug"
```
