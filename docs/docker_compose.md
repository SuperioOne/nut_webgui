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
