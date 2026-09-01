# Custom base path for reverse proxy

`nut_webgui` supports base-path overrides, which is useful if you're using
path-based routing between multiple services on the same hostname.

The base path can be configured in three ways:
- Via `config.toml`
    ```toml
    #... other configs

    [http_server]
    base_path = "uk/birmingham/nut"

    #... other configs
    ```
- Via the `NUTWG__HTTP_SERVER__BASE_PATH` environment variable.
- Via the `--base-path` CLI argument.

Supported base-path rules are:

- Multiple path segments are allowed (e.g. `segment1/segment2/...`).
- Leading/trailing whitespace and trailing forward slashes (`/`) are
  automatically trimmed. For example, `   base_path   /` and `/base_path/` are
  treated as the same.
- Percent-encoded characters are not supported.
- Unlike the RFC3986[^rfc3986] path definition, the `:` character is not
  allowed. However, you can still define some questionable base-paths such as
  `(-_-)/(@_@)/($_$)/nut-xyz.monitor/`

## Example case: `services/nut-web`

- Homepage -> `http://acme.com/services/nut-web/`
- Probes -> `http://acme.com/services/nut-web/probes/health`
- JSON API -> `http://acme.com/services/nut-web/api/ups`

**docker-compose.yaml**
```yaml
version: "3.3"
services:
  nutweb:
    image: codeberg.org/superiorone/nut_webgui:latest
    restart: always
    ports:
      - 9000:9000
    environment:
      UPSD_ADDR: "nut_server_address"
      UPSD_PORT: "3493"
      UPSD_USER: "admin"
      UPSD_PASS: "test"
      BASE_PATH: "services/nut-web"
```

### Example Caddy config

```nginx
:80 {
     reverse_proxy  /services/nut-web/* nutweb:9000
}
```

### Example Nginx config

```nginx
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

[^rfc3986]: [datatracker.ietf.org](https://datatracker.ietf.org/doc/html/rfc3986)
