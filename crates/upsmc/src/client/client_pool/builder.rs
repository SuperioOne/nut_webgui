use super::{ClientAllocator, NutPoolClient, server_addr::ServerAddr};
use crate::internal::item_pool::ItemPool;
use core::{num::NonZeroUsize, time::Duration};

pub struct NutPoolClientBuilder {
  allocator: ClientAllocator,
  limit: NonZeroUsize,
}

impl NutPoolClientBuilder {
  #[inline]
  pub fn new<T>(addr: T) -> Self
  where
    T: Into<ServerAddr>,
  {
    Self {
      limit: NonZeroUsize::new(1).unwrap(),
      allocator: ClientAllocator::new(addr.into()),
    }
  }

  #[inline]
  pub const fn with_timeout(mut self, timeout: Duration) -> Self {
    self.allocator.set_timeout(timeout);
    self
  }

  #[inline]
  pub const fn with_limit(mut self, limit: NonZeroUsize) -> Self {
    self.limit = limit;
    self
  }

  #[cfg(feature = "rustls")]
  #[inline]
  pub fn with_tls(
    mut self,
    srv_name: tokio_rustls::rustls::pki_types::ServerName<'static>,
    config: std::sync::Arc<tokio_rustls::rustls::ClientConfig>,
  ) -> Self {
    self.allocator.set_tls(srv_name, config);
    self
  }

  #[inline]
  pub fn open(self) -> NutPoolClient {
    NutPoolClient {
      pool: ItemPool::new(self.limit, self.allocator),
    }
  }
}
