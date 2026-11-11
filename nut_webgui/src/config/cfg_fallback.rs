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

    // Global ignore list is applied to every upsd after all layers are merged,
    // so it also covers upsd connections defined by env/CLI.
    for upsd in config.upsd.values_mut() {
      for ip in config.ignore_clients.iter() {
        if !upsd.ignore_clients.contains(ip) {
          upsd.ignore_clients.push(*ip);
        }
      }
    }

    Ok(config)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::cfg_toml::ServerTomlArgs;
  use core::net::IpAddr;

  fn ip(value: &str) -> IpAddr {
    value.parse().expect("test IP must be valid")
  }

  fn load(toml_content: &str) -> ServerConfig {
    let args: ServerTomlArgs = toml::from_str(toml_content).expect("test TOML must be valid");

    ServerConfig::new()
      .layer(args)
      .and_then(|config| config.layer(FallbackArgs))
      .expect("test config layers must apply")
  }

  #[test]
  fn global_ignore_list_applies_to_all_upsd() {
    let config = load(
      r#"
        ignore_clients = ["192.0.3.9"]

        [upsd.demo]
        address = "10.0.0.1"

        [upsd.reactor]
        address = "10.0.0.2"
      "#,
    );

    for upsd in config.upsd.values() {
      assert_eq!(upsd.ignore_clients, vec![ip("192.0.3.9")]);
    }
  }

  #[test]
  fn upsd_ignore_list_is_merged_with_global_list() {
    let config = load(
      r#"
        ignore_clients = ["192.0.3.9"]

        [upsd.demo]
        ignore_clients = ["192.0.2.10", "192.0.3.9"]

        [upsd.reactor]
        ignore_clients = ["192.0.2.11"]
      "#,
    );

    let demo = &config.upsd["demo"];
    assert_eq!(demo.ignore_clients, vec![ip("192.0.2.10"), ip("192.0.3.9")]);
    assert!(demo.is_client_ignored(&ip("192.0.2.10")));
    assert!(!demo.is_client_ignored(&ip("192.0.2.11")));

    let reactor = &config.upsd["reactor"];
    assert_eq!(
      reactor.ignore_clients,
      vec![ip("192.0.2.11"), ip("192.0.3.9")]
    );
    assert!(!reactor.is_client_ignored(&ip("192.0.2.10")));
  }

  #[test]
  fn global_ignore_list_applies_to_fallback_upsd() {
    let config = load(r#"ignore_clients = ["192.0.3.9"]"#);

    assert_eq!(
      config.upsd[DEFAULT_UPSD_KEY].ignore_clients,
      vec![ip("192.0.3.9")]
    );
  }

  #[test]
  fn invalid_ignore_ip_is_rejected() {
    let result = toml::from_str::<ServerTomlArgs>(r#"ignore_clients = ["not-an-ip"]"#);

    assert!(result.is_err());
  }
}
