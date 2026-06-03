# Docker Compose Examples

## Basic usage

Example topology:
```
┌──────┐
│ UPS1 ├──┐                                                  ┌─┐
└──────┘  │                                                  │C│
          │     ┌─────────────┐            ┌───────────┐     │L│
┌──────┐  │     │ NUT Service │    TCP     │  Docker   │     │I│
│ UPS2 ├──┼────►│             │◄──────────►│  Compose  │◄───►│E│
└──────┘  │     └─────────────┘            └───────────┘     │N│
          │     my-nut-srv:3493                              │T│
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
      - 80:1234                      # Expose container port 1234 on host port 80
    environment:
      POLL_FREQ: "60"
      POLL_INTERVAL: "5"
      UPSD_ADDR: "my-nut-srv"
      UPSD_PORT: "3493"
      UPSD_USER: "admin"
      UPSD_PASS: "test"
      LISTEN: "0.0.0.0"
      PORT: "1234"                   # Internal port the server listens on
      LOG_LEVEL: "debug"
    volumes:                         # Optional: Mount a volume to persist configuration.
      - config-data:/etc/nut_webgui

volumes:
  config-data:
```

## Same host

Example topology:
```
┌──────┐
│ UPS1 ├──┐                         ┌─┐
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

This setup is for when `nut_webgui` and the NUT service are running on the host
network space. Using `network_mode: host` allows `nut_webgui` to connect to the
NUT service via `localhost`.

**docker-compose.yaml**
```yaml
version: "3.3"
services:
  nutweb:
    image: ghcr.io/superioone/nut_webgui:latest
    restart: always
    network_mode: host       # Use the host's network stack.
    environment:
      POLL_FREQ: "60"
      POLL_INTERVAL: "5"
      UPSD_ADDR: "localhost"
      UPSD_PORT: "3493"
      UPSD_USER: "admin"
      UPSD_PASS: "test"
      PORT: "80"             # The container will listen on port 80 of the host.
      LOG_LEVEL: "debug"
```

## Using Docker secrets

[Docker secrets](https://docs.docker.com/compose/how-tos/use-secrets/) can be
used for managing sensitive informations like passwords and other configuration
files.

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
