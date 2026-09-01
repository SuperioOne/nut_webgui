# config.toml

`config.toml` has the least priority, but it is recommended to use the config
file as a baseline configuration and use environment variables and command-line
arguments to override settings when needed.

```toml
version = "1"
log_level = "info"
default_theme = "tokyo-night"

[http_server]
base_path = "/"
listen = "0.0.0.0"
port = 9000
worker_count = 8

[upsd.default]
username = "admin"
password = "where an old man of Aran goes around and around"
address = "localhost"
port = 3493
max_connection = 4
poll_freq = 30
poll_interval = 2
tls_mode = "disable"

[upsd.reactor]
address = "10.0.12.10"
username = "observer"
password = "AbsoluteSecurity"

[auth]
users_file = "/etc/nut_webgui/users.toml"
allow_anonymous_metrics = false
```
