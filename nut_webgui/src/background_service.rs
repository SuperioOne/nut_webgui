use core::{pin::Pin, time::Duration};
use futures::future::try_join_all;
use tokio::{
  task::{AbortHandle, JoinHandle},
  time::{error::Elapsed, timeout},
};
use tokio_util::sync::CancellationToken;

/// Trait for services that can be run in the background.
///
/// This trait defines the interface for services that need to run continuously
/// in the background until explicitly cancelled.
pub trait BackgroundService {
  /// Runs the service with a cancellation token.
  ///
  /// The cancellation token allows shutting down the service gracefully when needed.
  fn run(&self, token: CancellationToken)
  -> Pin<Box<dyn core::future::Future<Output = ()> + Send>>;
}

/// A runner for managing background services.
///
/// This struct manages multiple background services and provides facilities
/// for cancelling them all at once, as well as setting a timeout for shutdown operations.
pub struct BackgroundServiceRunner {
  cancellation: Option<CancellationToken>,
  wait_timeout: Option<Duration>,
  services: Vec<Box<dyn BackgroundService>>,
}

/// Handle for managing running background services.
pub struct RunnerHandle {
  cancellation: CancellationToken,
  wait_timeout: Duration,
  handles: Vec<(JoinHandle<()>, AbortHandle)>,
}

impl BackgroundServiceRunner {
  /// Creates a new `BackgroundServiceRunner`.
  ///
  /// This is the initial constructor that creates an empty runner with no services,
  /// cancellation token, or timeout set.
  pub const fn new() -> Self {
    Self {
      services: Vec::new(),
      cancellation: None,
      wait_timeout: None,
    }
  }

  /// Sets a maximum timeout duration for waiting on service shutdown.
  #[inline]
  pub const fn with_max_timeout(mut self, timeout: Duration) -> Self {
    self.wait_timeout = Some(timeout);
    self
  }

  /// Adds a new background service to the runner.
  ///
  /// The service will be started when `start()` is called and will run until
  /// cancelled or shut down via timeout.
  #[inline]
  pub fn add_service<T>(mut self, service: T) -> Self
  where
    T: BackgroundService + 'static,
  {
    self.services.push(Box::new(service));
    self
  }

  /// Starts all registered background services and returns a handle to manage them.
  ///
  /// This will spawn all services in background and return a handle that can be used
  /// to stop them later.
  pub fn start(self) -> RunnerHandle {
    let token = self
      .cancellation
      .unwrap_or_else(|| CancellationToken::new());

    let handles = self
      .services
      .into_iter()
      .map(|t| {
        // NOTE: Background service's `run(..)` can be called multiple times.
        // It's not required for now, but restart on failure can be implemented here.
        let future = t.run(token.clone());
        let service_handle = tokio::spawn(future);
        let abort_handle = service_handle.abort_handle();

        (service_handle, abort_handle)
      })
      .collect();

    RunnerHandle {
      cancellation: token,
      wait_timeout: self.wait_timeout.unwrap_or_else(|| Duration::from_secs(60)),
      handles,
    }
  }
}

impl RunnerHandle {
  /// Stops all running background services and waits for them to shut down.
  ///
  /// This method cancels all services using the associated cancellation token
  /// and waits for them to finish gracefully. If any service does not shut down
  /// within the specified timeout, it will be aborted.
  pub async fn stop(self) -> Result<(), Elapsed> {
    let unified_handle = try_join_all(self.handles.into_iter().map(
      |(handle, abort_handle)| async move {
        match timeout(self.wait_timeout, handle).await {
          Ok(_) => Ok(()),
          Err(err) => {
            abort_handle.abort();
            Err(err)
          }
        }
      },
    ));

    self.cancellation.cancel();
    _ = unified_handle.await?;

    Ok(())
  }
}
