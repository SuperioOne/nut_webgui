use super::{ConfigLayer, DEFAULT_UPSD_KEY, ServerConfig, UpsdConfig, error::ConfigError};

/// Applies fallbacks for required configurations
#[derive(Clone, Copy, Debug)]
pub struct FallbackArgs;

impl ConfigLayer for FallbackArgs {
  fn apply_layer(self, mut config: ServerConfig) -> Result<ServerConfig, ConfigError> {
    if config.upsd.is_empty() {
      let mut default_upsd = UpsdConfig::default();

      if !config.ignore_clients.is_empty() {
        default_upsd.ignore_clients = config.ignore_clients.clone();
      }

      config.upsd.insert(DEFAULT_UPSD_KEY.into(), default_upsd);
    }

    Ok(config)
  }
}
