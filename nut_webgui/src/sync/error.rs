use upsmc::UpsName;

#[derive(Debug)]
pub enum SyncTaskError {
  ClientError { inner: upsmc::error::Error },
  DeviceLoadFailed,
}

#[derive(Debug)]
pub struct DeviceLoadError {
  pub inner: upsmc::error::Error,
  pub name: UpsName,
}

impl std::fmt::Display for SyncTaskError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      SyncTaskError::ClientError { inner } => inner.fmt(f),
      SyncTaskError::DeviceLoadFailed => f.write_str("unable to get device details from upsd"),
    }
  }
}

impl std::fmt::Display for DeviceLoadError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!(
      "device load failed for {name}",
      name = self.name
    ))
  }
}

impl From<upsmc::error::Error> for SyncTaskError {
  fn from(value: upsmc::error::Error) -> Self {
    Self::ClientError { inner: value }
  }
}

impl std::error::Error for SyncTaskError {}
impl std::error::Error for DeviceLoadError {}
