# Multiple NUT Connection

Connections to multiple NUT servers can be configured in `config.toml` using the
`upsd.<namespace>` syntax.

**config.toml**
```toml
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

# Connection with default values
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

Exact same `config.toml` with inline tables:
```toml
version = "1"

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

After creating `config.toml`, mount it into the container.

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

- Namespaces must be unique.
- UPS devices are always referred to by their namespace, allowing the same UPS
  name to exist across different namespaces (e.g., `ups@kongou` and `ups@hiei`).
- The same NUT server address and port can be registered under multiple
  namespaces. This is kinda useless feature for 99.99% of users but still
  available as an option.
