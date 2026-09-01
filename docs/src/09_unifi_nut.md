# UniFi NUT

The current UniFi NUT server supports up to 3 client connections, and when this
limit is exceeded, the server may restarts itself[^unifi]. To prevent this issue,
**set the maximum connection limit to 1** using one of the following methods:

1. Set the `NUTWG__UPSD__MAX_CONNECTION` environment variable if you are using
  a single NUT server.

```sh
docker run -p 9000:9000 \
  -e UPSD_ADDR="19.13.12.1" \
  -e UPSD_USER="admin" \
  -e UPSD_PASS="test" \
  -e NUTWG__UPSD__MAX_CONNECTION=1 \
  codeberg.org/superiorone/nut_webgui:latest
```

2. Set the `max_connection=1` in the `config.toml` file.
  
```toml
version = "1"

[upsd.unifi]
max_connection = 1
address = "19.13.12.1"
username = "admin"
password = "test"
port = 3493
poll_freq = 20
poll_interval = 2
```

[^unifi]: [UniFi - community.ui.com](https://community.ui.com/questions/UPS-Tower-stops-responding-with-multiple-Nut-clients-connected/7f8956dd-3e2e-434b-8907-2a408eefe33f)
