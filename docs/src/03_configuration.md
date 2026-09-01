# Configuration

`nut_webgui` can be configured via CLI args, environment variables, or a config
file. All configuration options are merged into a single unified config based on
their priority.

Config priority order, highest to lowest:
1. [CLI args](./03_01_cli_args.md)
2. [Environment variables](./03_02_env_variables.md)
3. [config.toml file](./03_03_config_file.md)
