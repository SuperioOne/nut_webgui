# Docker Compose

# External Host
```
┌──────┐
│ UPS1 ├──┐                                                  ┌─┐
└──────┘  │     Server A                   Server B          │C│
          │     ┌─────────────┐            ┌───────────┐     │L│
┌──────┐  │     │ NUT Service │    TCP     │  Docker   │     │I│
│ UPS2 ├──┼────►│             │◄──────────►│  Compose  │◄───►│E│
└──────┘  │     └─────────────┘            └───────────┘     │N│
          │       myhost:1234                                │T│
┌──────┐  │                                                  │S│
│ UPS3 ├──┘                                                  └─┘
└──────┘
```

**docker-compose.yaml**
```yaml
version: "3.3"
services:
  nutweb:
    image: ghcr.io/superioone/nut_webgui:latest
    restart: always
    ports:
      - 80:1234
    environment:
      POLL_FREQ: "60"
      POLL_INTERVAL: "5"
      UPSD_ADDR: "myhost"
      UPSD_PORT: "1234"
      UPSD_USER: "admin"
      UPSD_PASS: "test"
      LISTEN: "0.0.0.0"
      PORT: "1234"
      LOG_LEVEL: "debug"

# Add other services, reverse proxy of your choice etc.
```

# Same host
```
┌──────┐
│ UPS1 ├──┐     Single Server       ┌─┐
└──────┘  │     ┌─────────────┐     │C│
          │     │   Docker    │     │L│
┌──────┐  │     │   Compose   │     │I│
│ UPS2 ├──┼────►├─────────────┤◄───►│E│
└──────┘  │     │ NUT Service │     │N│
          │     │             │     │T│
┌──────┐  │     └─────────────┘     │S│
│ UPS3 ├──┘     localhost:3493      └─┘
└──────┘

```

**docker-compose.yaml**
```yaml
version: "3.3"
services:
  nutweb:
    image: ghcr.io/superioone/nut_webgui:latest
    restart: always
    network_mode: host         # Share same host
    environment:
      POLL_FREQ: "60"
      POLL_INTERVAL: "5"
      UPSD_ADDR: "localhost"
      UPSD_PORT: "3493"
      UPSD_USER: "admin"
      UPSD_PASS: "test"
      PORT: "80"               # Outgoing port 
      LOG_LEVEL: "debug"
```

# Custom base path

Use `BASE_PATH` environment variable if you use path-base routing between multiple services.

**docker-compose.yaml**
```yaml
version: "3.3"
services:
  nutweb:
    image: ghcr.io/superioone/nut_webgui:latest
    restart: always
    environment:
      POLL_FREQ: "60"
      POLL_INTERVAL: "5"
      UPSD_ADDR: "nut_server_address"
      UPSD_PORT: "3493"
      UPSD_USER: "admin"
      UPSD_PASS: "test"
      LOG_LEVEL: "debug"
      BASE_PATH: "services/nut-web"
```

- Homepage -> `http://<host-name>/services/nut-web/`
- Health probe -> `http://<host-name>/services/nut-web/probes/health`
- JSON api -> `http://<host-name>/services/nut-web/api/ups`

## Caddy example config

```
:80 {
     reverse_proxy  /services/nut-web/* nutweb:9000
}
```

## Nginx example config

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
