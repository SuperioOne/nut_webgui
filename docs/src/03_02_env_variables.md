# Environment variables

Environment variables have the second-highest priority in configuration
settings. They accept direct values or file paths (file contents are read automatically).

## General

|Name                                  |Value Type                               |Description                                                                                         |
|--------------------------------------|-----------------------------------------|----------------------------------------------------------------------------------------------------|
|`NUTWG__CONFIG_FILE`                  |File path                                |Custom `config.toml` file path.                                                                     |
|`NUTWG__DEFAULT_THEME`                |Theme name                               |Web UI default theme.                                                                               |
|`NUTWG__LOG_LEVEL`                    |`error`, `warn`, `info`, `debug`, `trace`|Log level.                                                                                          |
|`NUTWG__SERVER_KEY`                   |Text                                     |Server signing key used for signing session tokens.                                                 |
|`NUTWG__AUTH__ALLOW_ANONYMOUS_METRICS`|`true` `1` `false` `0`                   |Allow access to the `/metrics` endpoint without an API key; ignored when authentication is disabled.|
|`NUTWG__AUTH__USERS_FILE`             |File path                                |Enable authentication with the provided `users.toml` file.                                          |
|`NUTWG__HTTP_SERVER__BASE_PATH`       |URI path                                 |Override HTTP server base path.                                                                     |
|`NUTWG__HTTP_SERVER__LISTEN`          |IPv4, IPv6                               |HTTP server listen address.                                                                         |
|`NUTWG__HTTP_SERVER__PORT`            |1-65535                                  |HTTP server listen port.                                                                            |
|`NUTWG__HTTP_SERVER__WORKER_COUNT`    |1-usize::MAX                             |HTTP server worker count.                                                                           |

## Default UPSD

If you only connect to a single NUT server and want to keep configurations
as simple as possible, connection details can be configured via `NUTWG__UPSD__*`
environment variables.

|Name                         |Value Type                 |Description                                                      |
|-----------------------------|---------------------------|-----------------------------------------------------------------|
|`NUTWG__UPSD__ADDRESS`       |IPv6, IPv4, hostname       |UPSD address.                                                    |
|`NUTWG__UPSD__MAX_CONNECTION`|1-usize::MAX               |Allowed maximum connections for the UPSD client.                 |
|`NUTWG__UPSD__NAME`          |Text                       |Target namespace for the `NUTWG__UPSD__*` environment variables. |
|`NUTWG__UPSD__PASSWORD`      |Text                       |UPSD password.                                                   |
|`NUTWG__UPSD__POLL_FREQ`     |1-u64::MAX                 |Non-critical UPS variables update frequency in seconds.          |
|`NUTWG__UPSD__POLL_INTERVAL` |1-u64::MAX                 |Critical UPS variables (`ups.status`) update interval in seconds.|
|`NUTWG__UPSD__PORT`          |1-65535                    |UPSD connection port.                                            |
|`NUTWG__UPSD__TLS_MODE`      |`strict`, `disable`, `skip`|Configures TLS communication between UPSD and the client.        |
|`NUTWG__UPSD__USERNAME`      |Text                       |UPSD username.                                                   |
