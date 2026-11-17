use self::{
  alloc::ClientAllocator, builder::NutPoolClientBuilder, marker::ClientStream,
  server_addr::ServerAddr,
};
use super::AsyncNutClient;
use crate::{
  CmdName, UpsName, VarName,
  client::NutClient,
  error::{Error, ErrorKind},
  internal::item_pool::{ItemPool, ItemPoolError},
  response,
};

mod alloc;

pub mod builder;
pub mod server_addr;

pub(crate) mod marker {
  use tokio::io::{AsyncRead, AsyncWrite};

  pub trait ClientStream: AsyncRead + AsyncWrite + Send + Sync + Unpin {}
  impl<A> ClientStream for A where A: AsyncWrite + AsyncRead + Send + Sync + Unpin {}
}

#[derive(Clone)]
pub struct NutPoolClient {
  pool: ItemPool<ClientAllocator>,
}

impl NutPoolClient {
  #[inline]
  pub fn new<T>(addr: T) -> NutPoolClientBuilder
  where
    T: Into<ServerAddr>,
  {
    NutPoolClientBuilder::new(addr)
  }

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
          ErrorKind::IOError { .. } | ErrorKind::EmptyResponse | ErrorKind::ConnectionTimeout => {
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
          ErrorKind::IOError { .. } | ErrorKind::ConnectionPoolClosed | ErrorKind::EmptyResponse | ErrorKind::ConnectionTimeout => {
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
  async fn get_cmd_desc<N, C>(self, ups: N, cmd: C) -> Result<response::CmdDesc, Error>
  where
    N: std::borrow::Borrow<UpsName>,
    C: std::borrow::Borrow<CmdName>,
  {
    impl_pooled_call!(self.pool, get_cmd_desc, ups.borrow(), cmd.borrow())
  }

  async fn get_protver(self) -> Result<response::ProtVer, Error> {
    impl_pooled_call!(self.pool, get_protver)
  }

  async fn get_ups_desc<N>(self, ups: N) -> Result<response::UpsDesc, Error>
  where
    N: std::borrow::Borrow<UpsName>,
  {
    impl_pooled_call!(self.pool, get_ups_desc, ups.borrow())
  }

  async fn get_var<N, V>(self, ups: N, var: V) -> Result<response::UpsVar, Error>
  where
    N: std::borrow::Borrow<UpsName>,
    V: std::borrow::Borrow<VarName>,
  {
    impl_pooled_call!(self.pool, get_var, ups.borrow(), var.borrow())
  }

  async fn get_var_type<N, V>(self, ups: N, var: V) -> Result<response::UpsVarType, Error>
  where
    N: std::borrow::Borrow<UpsName>,
    V: std::borrow::Borrow<VarName>,
  {
    impl_pooled_call!(self.pool, get_var_type, ups.borrow(), var.borrow())
  }

  async fn get_var_desc<N, V>(self, ups: N, var: V) -> Result<response::UpsVarDesc, Error>
  where
    N: std::borrow::Borrow<UpsName>,
    V: std::borrow::Borrow<VarName>,
  {
    impl_pooled_call!(self.pool, get_var_desc, ups.borrow(), var.borrow())
  }

  async fn get_ver(self) -> Result<response::DaemonVer, Error> {
    impl_pooled_call!(self.pool, get_ver)
  }

  async fn list_client<N>(self, ups: N) -> Result<response::ClientList, Error>
  where
    N: std::borrow::Borrow<UpsName>,
  {
    impl_pooled_call!(self.pool, list_client, ups.borrow())
  }

  async fn list_cmd<N>(self, ups: N) -> Result<response::CmdList, Error>
  where
    N: std::borrow::Borrow<UpsName>,
  {
    impl_pooled_call!(self.pool, list_cmd, ups.borrow())
  }

  async fn list_enum<N, V>(self, ups: N, var: V) -> Result<response::EnumList, Error>
  where
    N: std::borrow::Borrow<UpsName>,
    V: std::borrow::Borrow<VarName>,
  {
    impl_pooled_call!(self.pool, list_enum, ups.borrow(), var.borrow())
  }

  async fn list_range<N, V>(self, ups: N, var: V) -> Result<response::RangeList, Error>
  where
    N: std::borrow::Borrow<UpsName>,
    V: std::borrow::Borrow<VarName>,
  {
    impl_pooled_call!(self.pool, list_range, ups.borrow(), var.borrow())
  }

  async fn list_rw<N>(self, ups: N) -> Result<response::RwList, Error>
  where
    N: std::borrow::Borrow<UpsName>,
  {
    impl_pooled_call!(self.pool, list_rw, ups.borrow())
  }

  async fn list_ups(self) -> Result<response::UpsList, Error> {
    impl_pooled_call!(self.pool, list_ups)
  }

  async fn list_var<N>(self, ups: N) -> Result<response::UpsVarList, Error>
  where
    N: std::borrow::Borrow<UpsName>,
  {
    impl_pooled_call!(self.pool, list_var, ups.borrow())
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
