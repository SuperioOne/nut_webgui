use super::{AsyncNutClient, NutClient};
use crate::{
  CmdName, UpsName, VarName,
  errors::{Error, ErrorKind},
  internal::item_pool::{ItemAllocator, ItemPool, ItemPoolError},
  responses,
};
use core::{num::NonZeroUsize, time::Duration};
use std::net::ToSocketAddrs;
use tokio::net::TcpStream;
use tracing::warn;

pub struct ClientAllocator<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  addr: A,
  timeout: Option<Duration>,
}

impl<A> ItemAllocator for ClientAllocator<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  type Output = NutClient<TcpStream>;
  type Error = Error;

  async fn init(&self) -> Result<Self::Output, Self::Error> {
    let addr: Vec<_> = self.addr.to_socket_addrs()?.collect();
    let mut client = NutClient::connect(addr.as_slice()).await?;

    if let Some(timeout) = self.timeout {
      client.set_timeout(timeout);
    }

    Ok(client)
  }

  async fn dealloc(&self, item: Self::Output) {
    if let Err(err) = item.close().await {
      warn!(message = "unable to close a connection in pool", error = %err);
    }
  }

  #[inline]
  fn is_valid_state(&self, item: &mut Self::Output) -> impl Future<Output = bool> {
    item.is_open()
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

pub struct NutPoolClient<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  pool: ItemPool<NutClient<TcpStream>, ClientAllocator<A>>,
}

impl<A> Clone for NutPoolClient<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  #[inline]
  fn clone(&self) -> Self {
    Self {
      pool: self.pool.clone(),
    }
  }
}

unsafe impl<A> Send for NutPoolClient<A> where A: ToSocketAddrs + Send + Sync + 'static {}
unsafe impl<A> Sync for NutPoolClient<A> where A: ToSocketAddrs + Send + Sync + 'static {}

macro_rules! impl_pooled_call {
  ($pool:expr, $fn:ident $( , $($args:expr),+ )?) => {{
    let mut client = $pool.get().await?;

    match impl_pooled_call!(@action client, $fn $(, $($args),+)?) {
      Ok(res) => Ok(res),
      Err(err) => {
        match err.kind() {
          ErrorKind::IOError { .. } | ErrorKind::EmptyResponse | ErrorKind::RequestTimeout => {
            let mut client = $pool.get_checked().await?;
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

impl<A> NutPoolClient<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  pub fn new(addr: A, limit: NonZeroUsize) -> Self {
    Self {
      pool: ItemPool::new(
        limit,
        ClientAllocator {
          addr,
          timeout: None,
        },
      ),
    }
  }

  pub fn new_with_timeout(addr: A, limit: NonZeroUsize, timeout: Duration) -> Self {
    Self {
      pool: ItemPool::new(
        limit,
        ClientAllocator {
          addr,
          timeout: Some(timeout),
        },
      ),
    }
  }

  #[inline]
  pub fn close(self) -> impl Future<Output = ()> {
    self.pool.close()
  }

  #[inline]
  pub fn clear(&mut self) -> impl Future<Output = ()> {
    self.pool.clear()
  }
}

impl<A> AsyncNutClient for &NutPoolClient<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  async fn get_cmd_desc<N, C>(self, ups: N, cmd: C) -> Result<responses::CmdDesc, Error>
  where
    N: std::borrow::Borrow<UpsName>,
    C: std::borrow::Borrow<CmdName>,
  {
    impl_pooled_call!(&self.pool, get_cmd_desc, ups.borrow(), cmd.borrow())
  }

  async fn get_protver(self) -> Result<responses::ProtVer, Error> {
    impl_pooled_call!(&self.pool, get_protver)
  }

  async fn get_ups_desc<N>(self, ups: N) -> Result<responses::UpsDesc, Error>
  where
    N: std::borrow::Borrow<UpsName>,
  {
    impl_pooled_call!(&self.pool, get_ups_desc, ups.borrow())
  }

  async fn get_var<N, V>(self, ups: N, var: V) -> Result<responses::UpsVar, Error>
  where
    N: std::borrow::Borrow<UpsName>,
    V: std::borrow::Borrow<VarName>,
  {
    impl_pooled_call!(&self.pool, get_var, ups.borrow(), var.borrow())
  }

  async fn get_var_type<N, V>(self, ups: N, var: V) -> Result<responses::UpsVarType, Error>
  where
    N: std::borrow::Borrow<UpsName>,
    V: std::borrow::Borrow<VarName>,
  {
    impl_pooled_call!(&self.pool, get_var_type, ups.borrow(), var.borrow())
  }

  async fn get_var_desc<N, V>(self, ups: N, var: V) -> Result<responses::UpsVarDesc, Error>
  where
    N: std::borrow::Borrow<UpsName>,
    V: std::borrow::Borrow<VarName>,
  {
    impl_pooled_call!(&self.pool, get_var_desc, ups.borrow(), var.borrow())
  }

  async fn get_ver(self) -> Result<responses::DaemonVer, Error> {
    impl_pooled_call!(&self.pool, get_ver)
  }

  async fn list_client<N>(self, ups: N) -> Result<responses::ClientList, Error>
  where
    N: std::borrow::Borrow<UpsName>,
  {
    impl_pooled_call!(&self.pool, list_client, ups.borrow())
  }

  async fn list_cmd<N>(self, ups: N) -> Result<responses::CmdList, Error>
  where
    N: std::borrow::Borrow<UpsName>,
  {
    impl_pooled_call!(&self.pool, list_cmd, ups.borrow())
  }

  async fn list_enum<N, V>(self, ups: N, var: V) -> Result<responses::EnumList, Error>
  where
    N: std::borrow::Borrow<UpsName>,
    V: std::borrow::Borrow<VarName>,
  {
    impl_pooled_call!(&self.pool, list_enum, ups.borrow(), var.borrow())
  }

  async fn list_range<N, V>(self, ups: N, var: V) -> Result<responses::RangeList, Error>
  where
    N: std::borrow::Borrow<UpsName>,
    V: std::borrow::Borrow<VarName>,
  {
    impl_pooled_call!(&self.pool, list_range, ups.borrow(), var.borrow())
  }

  async fn list_rw<N>(self, ups: N) -> Result<responses::RwList, Error>
  where
    N: std::borrow::Borrow<UpsName>,
  {
    impl_pooled_call!(&self.pool, list_rw, ups.borrow())
  }

  async fn list_ups(self) -> Result<responses::UpsList, Error> {
    impl_pooled_call!(&self.pool, list_ups)
  }

  async fn list_var<N>(self, ups: N) -> Result<responses::UpsVarList, Error>
  where
    N: std::borrow::Borrow<UpsName>,
  {
    impl_pooled_call!(&self.pool, list_var, ups.borrow())
  }
}
