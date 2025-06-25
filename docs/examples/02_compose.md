# Docker Compose

## Basic usage

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
    volumes:                         # (optional) bind config directory to a volume
      - config-data:/etc/nut_webgui

volumes:
  config-data:

# Add other services, reverse proxy of your choice etc.
```

## Same host

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
    network_mode: host         # Use host network
    environment:
      POLL_FREQ: "60"
      POLL_INTERVAL: "5"
      UPSD_ADDR: "localhost"
      UPSD_PORT: "3493"
      UPSD_USER: "admin"
      UPSD_PASS: "test"
      PORT: "80"               # Outgoing port
      LOG_LEVEL: "debug"

volumes:
  config-data:
```

## With Secrets

All environment variables support loading values from files.

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
      UPSD_USER: "/run/secrets/upsd_username"
      UPSD_PASS: "/run/secrets/upsd_password"
      CONFIG_FILE: "/run/secrets/config_file"
    secrets:
      - upsd_username
      - upsd_password
      - config_file

secrets:
  upsd_username:
    file: ./upsd_user.txt
  upsd_password:
    file: ./upsd_password.txt
  config_file:
    file: ./other_configs.toml
```


