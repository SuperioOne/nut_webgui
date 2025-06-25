# Custom base path for reverse proxy

`nut_webgui` supports base path overrides, which is useful if you're using path-based routing between multiple services on the same hostname.

The base path can be configured in three ways:
- Via `config.toml`
    ```toml
    #... other configs

    [http_server]
    base_path = "uk/birmingham/nut"

    #... other configs
    ```
- Via environment variables: Set either `NUTWG__HTTP_SERVER__BASE_PATH` or `BASE_PATH`.
- Via CLI argumennt: Use the flag `--base-path 'uk/birmingham/nut'`

## Base path rules

- Allows multiple path segments (e.g. `segment1/segment2/...`).
- Whitespaces and trailing forward slashes (`/`) are trimmed. For example, `   base_path   /` and `/base_path/` are treated as the same.
- Percent encoded characters are not supported.
- Unlike RFC3986 path definition, the `:` character is not allowed. However, you can still define some questionable base paths such as `(-_-)/(@_@)/($_$)/nut-xyz.monitor/`


## Example case: `services/nut-web` as base path

- Homepage -> `http://<host-name>/services/nut-web/`
- Probes -> `http://<host-name>/services/nut-web/probes/health`
- JSON API -> `http://<host-name>/services/nut-web/api/ups`

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
      UPSD_ADDR: "nut_server_address"
      UPSD_PORT: "3493"
      UPSD_USER: "admin"
      UPSD_PASS: "test"
      LOG_LEVEL: "info"
      BASE_PATH: "services/nut-web"
```


### Caddy example config

```
:80 {
     reverse_proxy  /services/nut-web/* nutweb:9000
}
```

### Nginx example config

```
events {}

http {
    server {
        server_name   acme.com;
        listen        80;

        location /services/nut-web/ {
                proxy_pass         http://nutweb:9000;
        }
    }
}
```
