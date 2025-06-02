use super::{AsyncNutClient, NutClient};
use crate::{
  CmdName, UpsName, VarName,
  errors::{Error, ErrorKind},
  internal::item_pool::{ItemAllocator, ItemPool, ItemPoolError},
  responses,
};
use core::num::NonZeroUsize;
use std::net::ToSocketAddrs;
use tokio::net::TcpStream;
use tracing::warn;

pub struct ClientAllocator<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  addr: A,
}

impl<A> ItemAllocator for ClientAllocator<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  type Output = NutClient<TcpStream>;
  type Error = Error;

  async fn init(&self) -> Result<Self::Output, Self::Error> {
    let addr: Vec<_> = self.addr.to_socket_addrs()?.collect();
    let client = NutClient::connect(addr.as_slice()).await?;

    Ok(client)
  }

  async fn dealloc(&self, item: Self::Output) {
    if let Err(err) = item.close().await {
      warn!(message = "unable to close a connection in pool", error = %err);
    }
  }

  fn is_valid_state(&self, item: &mut Self::Output) -> impl Future<Output = bool> {
    item.is_open()
  }
}

impl From<ItemPoolError<Error>> for Error {
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
          ErrorKind::IOError { .. } | ErrorKind::EmptyResponse => {
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
          ErrorKind::IOError { .. } | ErrorKind::ConnectionPoolClosed | ErrorKind::EmptyResponse => {
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
      pool: ItemPool::new(limit, ClientAllocator { addr }),
    }
  }

  pub fn close(self) -> impl Future<Output = ()> {
    self.pool.close()
  }

  pub fn clear(&mut self) -> impl Future<Output = ()> {
    self.pool.clear()
  }
}

impl<A> AsyncNutClient for &NutPoolClient<A>
where
  A: ToSocketAddrs + Send + Sync + 'static,
{
  async fn get_cmd_desc(self, ups: &UpsName, cmd: &CmdName) -> Result<responses::CmdDesc, Error> {
    impl_pooled_call!(self.pool, get_cmd_desc, ups, cmd)
  }

  async fn get_protver(self) -> Result<responses::ProtVer, Error> {
    impl_pooled_call!(self.pool, get_protver)
  }

  async fn get_ups_desc(self, ups: &UpsName) -> Result<responses::UpsDesc, Error> {
    impl_pooled_call!(self.pool, get_ups_desc, ups)
  }

  async fn get_var(self, ups: &UpsName, var: &VarName) -> Result<responses::UpsVar, Error> {
    impl_pooled_call!(self.pool, get_var, ups, var)
  }

  async fn get_var_type(
    self,
    ups: &UpsName,
    var: &VarName,
  ) -> Result<responses::UpsVarType, Error> {
    impl_pooled_call!(self.pool, get_var_type, ups, var)
  }

  async fn get_var_desc(
    self,
    ups: &UpsName,
    var: &VarName,
  ) -> Result<responses::UpsVarDesc, Error> {
    impl_pooled_call!(self.pool, get_var_desc, ups, var)
  }

  async fn get_ver(self) -> Result<responses::DaemonVer, Error> {
    impl_pooled_call!(self.pool, get_ver)
  }

  async fn list_client(self, ups: &UpsName) -> Result<responses::ClientList, Error> {
    impl_pooled_call!(self.pool, list_client, ups)
  }

  async fn list_cmd(self, ups: &UpsName) -> Result<responses::CmdList, Error> {
    impl_pooled_call!(self.pool, list_cmd, ups)
  }

  async fn list_enum(self, ups: &UpsName, var: &VarName) -> Result<responses::EnumList, Error> {
    impl_pooled_call!(self.pool, list_enum, ups, var)
  }

  async fn list_range(self, ups: &UpsName, var: &VarName) -> Result<responses::RangeList, Error> {
    impl_pooled_call!(self.pool, list_range, ups, var)
  }

  async fn list_rw(self, ups: &UpsName) -> Result<responses::RwList, Error> {
    impl_pooled_call!(self.pool, list_rw, ups)
  }

  async fn list_ups(self) -> Result<responses::UpsList, Error> {
    impl_pooled_call!(self.pool, list_ups)
  }

  async fn list_var(self, ups: &UpsName) -> Result<responses::UpsVarList, Error> {
    impl_pooled_call!(self.pool, list_var, ups)
  }
}
