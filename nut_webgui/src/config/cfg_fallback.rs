use super::{ConfigLayer, DEFAULT_UPSD_KEY, ServerConfig, UpsdConfig, error::ConfigError};

/// Applies fallbacks for required configurations
#[derive(Clone, Copy, Debug)]
pub struct FallbackArgs;

impl ConfigLayer for FallbackArgs {
  fn apply_layer(self, mut config: ServerConfig) -> Result<ServerConfig, ConfigError> {
    if config.upsd.is_empty() {
      config
        .upsd
        .insert(DEFAULT_UPSD_KEY.into(), UpsdConfig::default());
    }

    Ok(config)
  }
}
