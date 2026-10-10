#[derive(Debug)]
pub struct AppInfo {
  pub version: &'static str,
  pub license: &'static str,
  pub repository: &'static str,
}

pub static APP_INFO: &'static AppInfo = &AppInfo {
  version: env!("CARGO_PKG_VERSION"),
  license: env!("CARGO_PKG_LICENSE"),
  repository: env!("CARGO_PKG_REPOSITORY"),
};
