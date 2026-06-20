# OpenMetric

`nut_webgui` can export UPS metrics in [OpenMetrics 1.0](https://prometheus.io/docs/specs/om/open_metrics_spec/#overview)
format. The device statistics are available on the `/metrics` endpoint.

## List of available metrics

The general metric naming syntax is:

```
nutwg_<UPS_VAR_NAME>[_<UNIT>]
```

|Name                               |Description              |
|-----------------------------------|-------------------------|
|nutwg_ambient_humidity             |Ambient humidity         |
|nutwg_ambient_temperature_celcius  |Ambient temperature      |
|nutwg_battery_charge               |Battery charge level     |
|nutwg_battery_current_amperes      |Battery current          |
|nutwg_battery_runtime_seconds      |Estimated battery runtime|
|nutwg_battery_temperature_celcius  |Battery temperature      |
|nutwg_battery_voltage_volts        |Battery voltage          |
|nutwg_input_bypass_current_amperes |Input bypass current     |
|nutwg_input_bypass_frequency_hertzs|Input bypass frequency   |
|nutwg_input_bypass_voltage_volts   |Input bypass voltage     |
|nutwg_input_count                  |Number of input sources  |
|nutwg_input_current_amperes        |Input current            |
|nutwg_input_frequency_hertz        |Input frequency          |
|nutwg_input_load                   |Input load percentage    |
|nutwg_input_power_voltamps         |Input power in VA        |
|nutwg_input_realpower_watts        |Input real power in Watt |
|nutwg_input_voltage_volts          |Input voltage            |
|nutwg_outlet_count                 |Number of outlets        |
|nutwg_outlet_current               |Outlet current           |
|nutwg_outlet_power_voltamps        |Outlet power in VA       |
|nutwg_outlet_realpower_watts       |Outlet real power in Watt|
|nutwg_outlet_voltage_volts         |Outlet voltage           |
|nutwg_output_current_amps          |Output current           |
|nutwg_output_frequency_hertzs      |Output frequency         |
|nutwg_output_power_percent         |Output power percentage  |
|nutwg_output_power_voltamps        |Output power in VA       |
|nutwg_output_realpower_watts       |Output real power in Watt|
|nutwg_output_voltage_volts         |Output voltage           |
|nutwg_ups_load                     |UPS load percentage      |
|nutwg_ups_output_voltage_volts     |UPS output voltage       |
|nutwg_ups_power_voltamps           |UPS power in VA          |
|nutwg_ups_realpower_watts          |UPS real power in Watt   |
|nutwg_ups_runtime_seconds          |UPS estimated runtime    |
|nutwg_ups_status                   |UPS status               |
|nutwg_ups_temperature_celcius      |UPS temperature          |

## Metric labels

Each metric includes `namespace` and `device` labels for filtering and alerting.

**Example metric with labels:**

```
# HELP nutwg_input_voltage_volts Input voltage
# TYPE nutwg_input_voltage_volts gauge
# UNIT nutwg_input_voltage_volts volts
nutwg_input_voltage_volts{namespace="hiei",device="tuncmatik_dg1200"} 212.0
nutwg_input_voltage_volts{namespace="kongou",device="tuncmatik_dg1200"} 210.0
nutwg_input_voltage_volts{namespace="hiei",device="apc_c1500"} 210.0
```

## Device Status

Device statuses are exported as `nutwg_ups_status`, with each flag encoded as a
separate label. If a status flag is active, its value is `1`; otherwise, it is
`0`.

`nut_webgui` uses status names defined in [RFC 9271](https://www.rfc-editor.org/rfc/rfc9271.html#symbols).
Some UPS drivers may have additional status names, but these are not currently
encoded in the UPS status metric.

**Example UPS with status "CHRG OL":**
```
# HELP nutwg_ups_status UPS status
# TYPE nutwg_ups_status gauge
nutwg_ups_status{status="CHRG",namespace="hiei",device="cyber_power_cp1500"} 1
nutwg_ups_status{status="OL",namespace="hiei",device="cyber_power_cp1500"} 1
nutwg_ups_status{status="ALARM",namespace="hiei",device="cyber_power_cp1500"} 0
nutwg_ups_status{status="BOOST",namespace="hiei",device="cyber_power_cp1500"} 0
nutwg_ups_status{status="BYPASS",namespace="hiei",device="cyber_power_cp1500"} 0
nutwg_ups_status{status="CAL",namespace="hiei",device="cyber_power_cp1500"} 0
nutwg_ups_status{status="COMM",namespace="hiei",device="cyber_power_cp1500"} 0
nutwg_ups_status{status="DISCHRG",namespace="hiei",device="cyber_power_cp1500"} 0
nutwg_ups_status{status="FSD",namespace="hiei",device="cyber_power_cp1500"} 0
nutwg_ups_status{status="LB",namespace="hiei",device="cyber_power_cp1500"} 0
nutwg_ups_status{status="NOCOMM",namespace="hiei",device="cyber_power_cp1500"} 0
nutwg_ups_status{status="OFF",namespace="hiei",device="cyber_power_cp1500"} 0
nutwg_ups_status{status="OB",namespace="hiei",device="cyber_power_cp1500"} 0
nutwg_ups_status{status="OVER",namespace="hiei",device="cyber_power_cp1500"} 0
nutwg_ups_status{status="RB",namespace="hiei",device="cyber_power_cp1500"} 0
nutwg_ups_status{status="TEST",namespace="hiei",device="cyber_power_cp1500"} 0
nutwg_ups_status{status="TICK",namespace="hiei",device="cyber_power_cp1500"} 0
nutwg_ups_status{status="TOCK",namespace="hiei",device="cyber_power_cp1500"} 0
nutwg_ups_status{status="TRIM",namespace="hiei",device="cyber_power_cp1500"} 0
```

## Example Prometheus config

There are no strict requirements for scrape intervals, but it is recommended to
set it identical to your `poll_freq` value.

```yaml
scrape_configs:
  - job_name: nut_service
    scrape_interval: 20s
    scrape_timeout: 5s
    scrape_protocols: ["OpenMetricsText1.0.0"]
    static_configs:
     - targets: ['localhost:9000']
    metrics_path: "/metrics"
    scheme: http
```

The `/metrics` endpoint also requires authentication via an API key when auth is
enabled. See [Enabling auth](./08_enabling_auth.md) for instructions on
generating API keys.

> Authentication on `/metrics` endpoint can be disabled via:
> - CLI argument: `--anonymous-metrics "true"`
> - environment variable: `NUTWG__AUTH__ALLOW_ANONYMOUS_METRICS="true"`
> - `config.toml`: `[auth].allow_anonymous_metrics = true`

After generating the API key, add authorization using the `Bearer` scheme to
your Prometheus configuration.

```yaml
scrape_configs:
  - job_name: nut_service
    scrape_interval: 30s
    scrape_timeout: 5s
    scrape_protocols: ["OpenMetricsText1.0.0"]
    static_configs:
     - targets: ['localhost:9000']
    metrics_path: "/metrics"
    scheme: http
    authorization:
      type: Bearer
      credentials: "SWYgb25seSBJIGNvdWxkIGJlIHNvIGluY2FuZGVzY2VudA=="
```

> For other config options see the Prometheus own [documentation.](https://prometheus.io/docs/prometheus/latest/configuration/configuration/#http_config)
