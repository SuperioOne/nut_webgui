use super::{ConfigLayer, DEFAULT_UPSD_KEY, ServerConfig, UpsdConfig, error::ConfigError};

/// Applies fallbacks for required configurations
#[derive(Clone, Copy, Debug)]
pub struct FallbackArgs;

impl ConfigLayer for FallbackArgs {
  fn apply_layer(self, mut config: ServerConfig) -> Result<ServerConfig, ConfigError> {
    let default_upsd = config
      .upsd
      .entry(DEFAULT_UPSD_KEY.into())
      .or_insert(UpsdConfig::default());

    if !config.ignore_clients.is_empty() {
      let mut filter = Vec::new();
      filter.extend(&default_upsd.ignore_clients);

      for ip in config.ignore_clients.iter() {
        if !filter.contains(ip) {
          filter.push(*ip);
        }
      }

      default_upsd.ignore_clients = filter.into_boxed_slice();
    }

    Ok(config)
  }
}
