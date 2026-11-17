use super::{marker::ClientStream, server_addr::ServerAddr};
use crate::{
  client::NutClient,
  error::{Error, ErrorKind},
  internal::item_pool::{ItemAllocator, ItemState},
};
use core::time::Duration;
use std::{net::ToSocketAddrs, pin::Pin, sync::Arc};
use tokio::time::timeout;
use tracing::warn;

#[cfg(feature = "rustls")]
use tokio_rustls::rustls::{ClientConfig, pki_types::ServerName};

#[cfg(feature = "rustls")]
#[derive(Debug)]
struct TlsConfig {
  srv_name: ServerName<'static>,
  config: Arc<ClientConfig>,
}

#[derive(Debug)]
pub struct ClientAllocator {
  addr: Arc<ServerAddr>,
  timeout: Option<Duration>,

  #[cfg(feature = "rustls")]
  tls_config: Option<TlsConfig>,
}

impl ClientAllocator {
  #[inline]
  pub fn new(addr: ServerAddr) -> Self {
    Self {
      addr: Arc::new(addr),
      timeout: None,

      #[cfg(feature = "rustls")]
      tls_config: None,
    }
  }

  #[cfg(feature = "rustls")]
  pub fn set_tls(&mut self, srv: ServerName<'static>, config: Arc<ClientConfig>) {
    self.tls_config = Some(TlsConfig {
      srv_name: srv,
      config,
    });
  }

  pub const fn set_timeout(&mut self, timeout: Duration) {
    self.timeout = Some(timeout);
  }

  async fn open_connection(&self) -> Result<Box<dyn ClientStream>, Error> {
    let addrs: Vec<_> = self.addr.to_socket_addrs()?.collect();

    #[cfg(feature = "rustls")]
    {
      if let Some(tls_config) = &self.tls_config {
        let connection: Box<dyn ClientStream> = Box::new(
          NutClient::connect_with_tls(
            addrs.as_slice(),
            tls_config.srv_name.clone(),
            tls_config.config.clone(),
          )
          .await?
          .into_inner(),
        );

        return Ok(connection);
      }
    }

    Ok(Box::new(
      NutClient::connect(addrs.as_slice()).await?.into_inner(),
    ))
  }
}

impl ItemAllocator for ClientAllocator {
  type Item = NutClient<Box<dyn ClientStream>>;
  type Error = Error;

  fn init(&self) -> Pin<Box<dyn Future<Output = Result<Self::Item, Self::Error>> + Send + '_>> {
    Box::pin(async move {
      let connection = {
        let future = self.open_connection();
        match self.timeout {
          Some(duration) => match timeout(duration, future).await {
            Ok(res) => res,
            Err(_) => Err(ErrorKind::ConnectionTimeout.into()),
          },
          None => future.await,
        }
      }?;

      let mut client = NutClient::from(connection);

      if let Some(timeout) = self.timeout {
        client.set_timeout(timeout);
      }

      Ok(client)
    })
  }

  fn dealloc(&self, item: Self::Item) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
    Box::pin(async move {
      if let Err(err) = item.close().await {
        warn!(message = "unable to close a tcp connection in pool", error = %err);
      }
    })
  }

  fn prealloc_check(
    &self,
    mut item: Self::Item,
  ) -> Pin<Box<dyn Future<Output = ItemState<Self::Item>> + Send + '_>> {
    Box::pin(async move {
      if item.is_open().await {
        ItemState::Ready(item)
      } else {
        ItemState::Destroy(item)
      }
    })
  }
}
