# Multiple NUT servers

Multiple NUT server connections can be defined in `config.toml` using the
`upsd.<namespace>` syntax.

Each connection is referred to as a unique `namespace`, and UPS devices
listed by the connection are referred to by their name and namespace
(e.g., `ups@kongou` and `ups@hiei`).

> [!NOTE]
> The same NUT server address and port can be registered under different
>  namespaces. This is a kinda useless feature for 99.99% of users, but is still
>  available as an option.

## Example config.toml

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

Exact same `config.toml` with inline table syntax:
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
