# Debugging

Clone the git repository.

```sh
git clone --recurse-submodules https://codeberg.org/SuperiorOne/nut_webgui.git
```

> [!IMPORTANT]
> `--recurse-submodules` flag is required for the UPS validation tests. It pulls
> NUT Device Dumps Library[^nut-ddl] as a submodule, which contains known UPS
> device dumps.

## Run server

- Start with bacon:
  `make watch` can start the development server. It simply calls 
  `bacon -j serve`[^bacon].

  ```bash
  # (Optional) Set your NUT test server configs.
  export NUTWG__CONFIG_FILE="test.config.toml"
  export NUTWG__LOG_LEVEL="trace"
  export NUTWG__UPSD__ADDRESS="10.0.0.1"
  export NUTWG__UPSD__USERNAME="cid"
  export NUTWG__UPSD__PASSWORD="i_am_atomic"

  make watch
  ```

- or use cargo: `cargo run -p nut_webgui`

## Tests

`make test` command runs all available tests.

## Simulating UPS devices and NUT server with containers

A basic NUT server container image is available in the
`test_tools/dummy_server` directory in the source code.
It starts a NUT server, and automatically configures test devices.

Build container image:

```sh
cd ./test_tools/dummy_server
docker build -t dummy_server:latest -f dummy_server.Dockerfile
```

Start NUT server
```sh
docker run --rm -p 3493:3493 dummy_server:latest
```

(Optional) Start with custom device dump files
```sh
docker run --rm -p 3493:3493 -v \
"$(pwd)/example-devices":/nut_devices dummy_server:latest
```

## Testing TLS

The dummy server is configured with a self-signed certificate.
The root certificate is available at `test_tools/dummy_server/root.crt`.

[^nut-ddl]: [Github - networkupstools/nut-ddl](https://github.com/networkupstools/nut-ddl)
[^bacon]: [Github - Canop/bacon](https://github.com/Canop/bacon)
