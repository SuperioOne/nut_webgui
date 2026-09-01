# Podman/Docker

**Quickstart:**
```sh
docker run -p 9000:9000 \
  -e UPSD_ADDR=10.0.0.1 \
  -e UPSD_USER=test \
  -e UPSD_PASS=strongpass \
  codeberg.org/superiorone/nut_webgui:latest
```


## Mounting a configuration file

Create a `config.toml` on your host and mount it into the container:

```sh
echo 'version = "1"

[upsd.default]
address = "10.0.0.1"
username = "admin"
password = "test"

' > config.toml;
```

Start `nut_webgui` with the `config.toml` file:

```sh
docker run \
  -p 9000:9000 \
  -v "$(pwd)/config.toml":"/etc/nut_webgui/config.toml" \
  codeberg.org/superiorone/nut_webgui:latest
```

## Mounting a directory as a volume

You can mount an empty directory to `/etc/nut_webgui` to persist configurations.

If no config exists at `/etc/nut_webgui`, container entrypoint generates one
automatically.

> [!CAUTION]
> When the container user is overridden using the `--user` option, the container
> will not automatically generate the default configuration file or server key.
> You are responsible for mounting the configuration files with the correct
> permissions and setting up any required environment variables.


```sh
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
  codeberg.org/superiorone/nut_webgui:latest
```

## Using CLI arguments

You can also override settings by passing arguments directly to the server
command.

```bash
docker run \
  -p 9000:9000 \
  codeberg.org/superiorone/nut_webgui:latest \
  nut_webgui --allow-env --log-level "debug"
```
