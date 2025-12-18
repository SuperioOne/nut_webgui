# Multiple NUT Connection

Multiple NUT server connection can be defined in `config.toml` using the `upsd.<namespace>` syntax.

**config.toml**
```toml
# (Required) set version to "1"
version = "1"

# Minimal (read-only mode, can't use instcmd, fsd, setvar)
[upsd.kongou]
address = "19.12.5.18"

# Minimal with upsd authentication (can use instcmd, fsd, setvar)
[upsd.hiei]
address = "19.12.11.21"
username = "admin"
password = "test"

# Complete configuration example
[upsd.kirishima]
address = "19.13.12.1"
username = "admin"
password = "test"
port = 4493
max_connection = 12
poll_freq = 10
poll_interval = 1
tls_mode = "strict"

# Connection with default configurations
[upsd.haruna]
## Default values are:
# address = "localhost"
# port = 3493
# max_connection = 4
# poll_freq = 30
# poll_interval = 2
# tls_mode = "disable"


## ... Other config.toml options ...
```

After creating the `config.toml`, it can be mounted into the container.


**docker/podman run**
```bash
docker run \
  -p 9000:9000 \
  -v "$(pwd)/config.toml":"/etc/nut_webgui/config.toml" \
  ghcr.io/superioone/nut_webgui:latest
```

or

**docker-compose.yaml**
```yaml
version: "3.3"
services:
  nutweb:
    image: ghcr.io/superioone/nut_webgui:latest
    restart: always
    ports:
      - 9000:9000
    environment:
      CONFIG_FILE: "/run/secrets/config_file"
    secrets:
      - config_file

secrets:
  config_file:
    file: ./config.toml
```

## Notes

- Config file uses TOML v1.1.0 format.
- Namespaces must be unique.
- UPS devices are always referred by their namespace, allowing the use of the same UPS name under different namespaces. For example: `ups@kongou` and `ups@hiei`.
- The same NUT server address and port can be added multiple times as different namespace. This is kinda useless feature for 99.99% of users but is available as an option.

  Example using inline tables:
  ```toml
  [upsd]
  kongou = { address = "19.12.1.17" }
  hiei = {
    address = "19.12.11.21",
    username = "admin",
    password = "test"
  }
  kirishima = {
    address = "19.13.12.1",
    username = "admin",
    password = "test",
    port = 4493,
    max_connection = 12,
    poll_freq = 10,
    poll_interval = 1,
    tls_mode = "strict"
  }
  haruna = {}
  ```

