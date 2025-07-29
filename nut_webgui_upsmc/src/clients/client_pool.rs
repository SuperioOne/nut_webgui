use super::AsyncNutClient;
use crate::{
  CmdName, UpsName, VarName,
  clients::NutClient,
  errors::{Error, ErrorKind},
  internal::item_pool::{ItemAllocator, ItemPool, ItemPoolError, ItemState},
  responses,
};
use core::num::NonZeroUsize;
use core::time::Duration;
use std::{
  net::{SocketAddr, ToSocketAddrs},
  pin::Pin,
  sync::Arc,
};
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::warn;

#[derive(Debug)]
pub enum ServerAddr {
  SocketAddr(SocketAddr),
  Host(String),
}

impl ToSocketAddrs for ServerAddr {
  type Iter = std::vec::IntoIter<SocketAddr>;

  fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
    match self {
      ServerAddr::SocketAddr(socket_addr) => Ok(vec![*socket_addr].into_iter()),
      ServerAddr::Host(host) => host.to_socket_addrs(),
    }
  }
}

impl From<String> for ServerAddr {
  #[inline]
  fn from(value: String) -> Self {
    Self::Host(value)
  }
}

impl From<SocketAddr> for ServerAddr {
  #[inline]
  fn from(value: SocketAddr) -> Self {
    Self::SocketAddr(value)
  }
}

impl From<ItemPoolError<Error>> for Error {
  #[inline]
  fn from(value: ItemPoolError<Error>) -> Self {
    match value {
      ItemPoolError::PoolClosed => ErrorKind::ConnectionPoolClosed.into(),
      ItemPoolError::AllocatorError { inner } => inner,
    }
  }
}

#[cfg(feature = "rustls")]
#[derive(Debug)]
struct TlsConfig {
  srv_name: tokio_rustls::rustls::pki_types::ServerName<'static>,
  config: Arc<tokio_rustls::rustls::ClientConfig>,
}

#[derive(Debug)]
struct ClientAllocator {
  addr: Arc<ServerAddr>,
  timeout: Option<Duration>,

  #[cfg(feature = "rustls")]
  tls_config: Option<TlsConfig>,
}

pub struct NutPoolClientBuilder {
  allocator: ClientAllocator,
  limit: NonZeroUsize,
}

impl NutPoolClientBuilder {
  #[inline]
  pub fn new(addr: ServerAddr) -> Self {
    Self {
      limit: NonZeroUsize::new(1).unwrap(),
      allocator: ClientAllocator {
        addr: Arc::new(addr),
        timeout: None,

        #[cfg(feature = "rustls")]
        tls_config: None,
      },
    }
  }

  #[inline]
  pub fn with_timeout(mut self, timeout: Duration) -> Self {
    self.allocator.timeout = Some(timeout);
    self
  }

  #[inline]
  pub fn with_limit(mut self, limit: NonZeroUsize) -> Self {
    self.limit = limit;
    self
  }

  #[cfg(feature = "rustls")]
  #[inline]
  pub fn with_tls(
    mut self,
    srv_name: tokio_rustls::rustls::pki_types::ServerName<'static>,
    config: Arc<tokio_rustls::rustls::ClientConfig>,
  ) -> Self {
    self.allocator.tls_config = Some(TlsConfig { srv_name, config });
    self
  }

  #[inline]
  pub fn build(self) -> NutPoolClient {
    NutPoolClient {
      pool: ItemPool::new(self.limit, self.allocator),
    }
  }
}

pub trait ClientStream: AsyncRead + AsyncWrite + Send + Sync + Unpin {}
impl<A> ClientStream for A where A: AsyncWrite + AsyncRead + Send + Sync + Unpin {}

impl ItemAllocator for ClientAllocator {
  type Item = NutClient<Box<dyn ClientStream>>;
  type Error = Error;

  fn init(&self) -> Pin<Box<dyn Future<Output = Result<Self::Item, Self::Error>> + Send + '_>> {
    Box::pin(async move {
      let addr_resolutions: Vec<_> = self.addr.to_socket_addrs()?.collect();
      let connection: Box<dyn ClientStream> = {
        #[cfg(feature = "rustls")]
        {
          if let Some(tls_config) = &self.tls_config {
            Box::new(
              NutClient::connect_with_tls(
                addr_resolutions.as_slice(),
                tls_config.srv_name.clone(),
                tls_config.config.clone(),
              )
              .await?
              .into_inner(),
            )
          } else {
            Box::new(
              NutClient::connect(addr_resolutions.as_slice())
                .await?
                .into_inner(),
            )
          }
        }

        #[cfg(not(feature = "rustls"))]
        {
          Box::new(
            NutClient::connect(addr_resolutions.as_slice())
              .await?
              .into_inner(),
          )
        }
      };

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

pub struct NutPoolClient {
  pool: ItemPool<ClientAllocator>,
}

impl Clone for NutPoolClient {
  #[inline]
  fn clone(&self) -> Self {
    Self {
      pool: self.pool.clone(),
    }
  }
}

impl NutPoolClient {
  #[inline]
  pub fn close(self) -> impl Future<Output = ()> {
    self.pool.close()
  }

  #[inline]
  pub fn clear(&mut self) -> impl Future<Output = ()> {
    self.pool.clear()
  }

  pub async fn get_client(&self) -> Result<NutClient<Box<dyn ClientStream>>, ItemPoolError<Error>> {
    let client = self.pool.get_checked().await?.into_inner();
    Ok(client)
  }
}

macro_rules! impl_pooled_call {
  ($pool:expr, $fn:ident $( , $($args:expr),+ )?) => {{
    let mut client = match $pool.get().await {
      Ok(c) => c,
      Err(err) => {return Err(err.into());}
    };

    match impl_pooled_call!(@action client, $fn $(, $($args),+)?) {
      Ok(res) => Ok(res),
      Err(err) => {
        match err.kind() {
          ErrorKind::IOError { .. } | ErrorKind::EmptyResponse | ErrorKind::RequestTimeout => {
            let mut client = match $pool.get_checked().await {
              Ok(c) => c,
              Err(err) => {return Err(err.into());}
            };

            impl_pooled_call!(@action client, $fn $(, $($args),+)?)
          }
          _ => Err(err)
        }
      }
    }
  }};

  (@action $client:expr, $fn:ident $( , $($args:expr),+ )?) => {{
    match $client.$fn($($($args),+)?).await {
      Ok(result) => {
        _ = $client.release().await;
        Ok(result)
      },
      Err(err) => {
        match err.kind() {
          ErrorKind::IOError { .. } | ErrorKind::ConnectionPoolClosed | ErrorKind::EmptyResponse | ErrorKind::RequestTimeout => {
          drop($client);
        }
          _ => {
            _ = $client.release().await;
          }
        };

        Err(err)
      }
    }
  }};
}

impl AsyncNutClient for &NutPoolClient {
  async fn get_cmd_desc<N, C>(self, ups: N, cmd: C) -> Result<responses::CmdDesc, Error>
  where
    N: std::borrow::Borrow<UpsName>,
    C: std::borrow::Borrow<CmdName>,
  {
    impl_pooled_call!(self.pool, get_cmd_desc, ups.borrow(), cmd.borrow())
  }

  async fn get_protver(self) -> Result<responses::ProtVer, Error> {
    impl_pooled_call!(self.pool, get_protver)
  }

  async fn get_ups_desc<N>(self, ups: N) -> Result<responses::UpsDesc, Error>
  where
    N: std::borrow::Borrow<UpsName>,
  {
    impl_pooled_call!(self.pool, get_ups_desc, ups.borrow())
  }

  async fn get_var<N, V>(self, ups: N, var: V) -> Result<responses::UpsVar, Error>
  where
    N: std::borrow::Borrow<UpsName>,
    V: std::borrow::Borrow<VarName>,
  {
    impl_pooled_call!(self.pool, get_var, ups.borrow(), var.borrow())
  }

  async fn get_var_type<N, V>(self, ups: N, var: V) -> Result<responses::UpsVarType, Error>
  where
    N: std::borrow::Borrow<UpsName>,
    V: std::borrow::Borrow<VarName>,
  {
    impl_pooled_call!(self.pool, get_var_type, ups.borrow(), var.borrow())
  }

  async fn get_var_desc<N, V>(self, ups: N, var: V) -> Result<responses::UpsVarDesc, Error>
  where
    N: std::borrow::Borrow<UpsName>,
    V: std::borrow::Borrow<VarName>,
  {
    impl_pooled_call!(self.pool, get_var_desc, ups.borrow(), var.borrow())
  }

  async fn get_ver(self) -> Result<responses::DaemonVer, Error> {
    impl_pooled_call!(self.pool, get_ver)
  }

  async fn list_client<N>(self, ups: N) -> Result<responses::ClientList, Error>
  where
    N: std::borrow::Borrow<UpsName>,
  {
    impl_pooled_call!(self.pool, list_client, ups.borrow())
  }

  async fn list_cmd<N>(self, ups: N) -> Result<responses::CmdList, Error>
  where
    N: std::borrow::Borrow<UpsName>,
  {
    impl_pooled_call!(self.pool, list_cmd, ups.borrow())
  }

  async fn list_enum<N, V>(self, ups: N, var: V) -> Result<responses::EnumList, Error>
  where
    N: std::borrow::Borrow<UpsName>,
    V: std::borrow::Borrow<VarName>,
  {
    impl_pooled_call!(self.pool, list_enum, ups.borrow(), var.borrow())
  }

  async fn list_range<N, V>(self, ups: N, var: V) -> Result<responses::RangeList, Error>
  where
    N: std::borrow::Borrow<UpsName>,
    V: std::borrow::Borrow<VarName>,
  {
    impl_pooled_call!(self.pool, list_range, ups.borrow(), var.borrow())
  }

  async fn list_rw<N>(self, ups: N) -> Result<responses::RwList, Error>
  where
    N: std::borrow::Borrow<UpsName>,
  {
    impl_pooled_call!(self.pool, list_rw, ups.borrow())
  }

  async fn list_ups(self) -> Result<responses::UpsList, Error> {
    impl_pooled_call!(self.pool, list_ups)
  }

  async fn list_var<N>(self, ups: N) -> Result<responses::UpsVarList, Error>
  where
    N: std::borrow::Borrow<UpsName>,
  {
    impl_pooled_call!(self.pool, list_var, ups.borrow())
  }
}
