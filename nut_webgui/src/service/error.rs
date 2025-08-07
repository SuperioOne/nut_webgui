use nut_webgui_upsmc::UpsName;

#[derive(Debug)]
pub(super) enum SyncTaskError {
  ClientError {
    inner: nut_webgui_upsmc::error::Error,
  },
  DeviceLoadFailed,
}

#[derive(Debug)]
pub(super) struct DeviceLoadError {
  pub inner: nut_webgui_upsmc::error::Error,
  pub name: UpsName,
}

#[derive(Debug)]
pub struct ShutdownTimedOut;

impl From<nut_webgui_upsmc::error::Error> for SyncTaskError {
  fn from(value: nut_webgui_upsmc::error::Error) -> Self {
    Self::ClientError { inner: value }
  }
}

impl std::fmt::Display for SyncTaskError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      SyncTaskError::ClientError { inner } => inner.fmt(f),
      SyncTaskError::DeviceLoadFailed => f.write_str("unable to get new device details from upsd"),
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

impl std::fmt::Display for ShutdownTimedOut {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!(
      "graceful shutdown timed out; services closed forcefully",
    ))
  }
}

pub(super) trait IntoLoadError<T> {
  fn map_load_err(self, name: &UpsName) -> Result<T, DeviceLoadError>;
}

impl<T> IntoLoadError<T> for Result<T, nut_webgui_upsmc::error::Error> {
  fn map_load_err(self, name: &UpsName) -> Result<T, DeviceLoadError> {
    match self {
      Ok(val) => Ok(val),
      Err(err) => Err(DeviceLoadError {
        inner: err,
        name: name.to_owned(),
      }),
    }
  }
}

impl std::error::Error for SyncTaskError {}
impl std::error::Error for DeviceLoadError {}
impl std::error::Error for ShutdownTimedOut {}
