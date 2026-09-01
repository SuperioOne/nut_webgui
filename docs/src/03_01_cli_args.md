# CLI args

CLI arguments hold the highest priority in configuration settings.

|Argument flag        |Description                                                                                    |
|---------------------|-----------------------------------------------------------------------------------------------|
|`--allow-env`        |Allow the application to load configuration from environment variables.                         |
|`--anonymous-metrics`|Set anonymous access for the `/metrics` endpoint, ignored when auth is disabled. Default is `false`.|
|`--base-path`        |Override HTTP server base path. Default is `/`.                                                |
|`--config-file`      |Path to the config.toml file.                                                                  |
|`--default-theme`    |Web UI default theme.                                                                          |
|`--listen`           |Listen address for the HTTP server. Default is `0.0.0.0`.                                      |
|`--log-level`        |Log level for the HTTP server. Default is `info`.                                              |
|`--port`             |Port used by the HTTP server. Default is `9000`.                                               |
|`--server-key`       |Set private server key value. Default is auto-generated value.                                 |
|`--with-auth`        |Enable authentication with the `user.toml` file.                                               |
|`--worker-count`     |Set HTTP server worker count.                                                                  |
