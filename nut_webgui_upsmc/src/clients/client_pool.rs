use tokio::net::TcpStream;
use tracing::warn;

use super::{AsyncNutClient, NutClient};
use crate::{
  CmdName, UpsName, VarName,
  errors::{Error, ErrorKind},
  internal::item_pool::{ItemAllocator, ItemPool, ItemPoolError},
  responses,
};
use core::{net::SocketAddr, num::NonZeroUsize};

pub struct ClientAllocator {
  addr: SocketAddr,
}

impl ItemAllocator for ClientAllocator {
  type Output = NutClient<TcpStream>;
  type Error = Error;

  fn init(&self) -> impl Future<Output = Result<Self::Output, Self::Error>> {
    NutClient::connect(self.addr)
  }

  async fn dealloc(&self, item: Self::Output) {
    if let Err(err) = item.close().await {
      warn!(message = "unable to close a connection in pool", error = %err);
    }
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

pub struct NutPoolClient {
  pool: ItemPool<NutClient<TcpStream>, ClientAllocator>,
}

unsafe impl Send for NutPoolClient {}
unsafe impl Sync for NutPoolClient {}

macro_rules! impl_pooled_call {
  ($pool:expr, $fn:ident $( , $($args:expr),+ )?) => {{
    let mut client = $pool.get().await?;

    match client.$fn($($($args),+)?).await {
      Ok(result) => {
        _ = client.release().await;
        Ok(result)
      },
      Err(err) => {
        match err.kind() {
          ErrorKind::IOError { .. } => {}
          ErrorKind::ConnectionPoolClosed => {}
          _ => {
            _ = client.release().await;
          }
        };

        Err(err)
      }
    }
  }};
}

impl NutPoolClient {
  pub fn new(addr: SocketAddr, limit: NonZeroUsize) -> Self {
    Self {
      pool: ItemPool::new(limit, ClientAllocator { addr }),
    }
  }

  pub async fn close(self) {
    self.pool.close().await;
  }
}

impl AsyncNutClient for &NutPoolClient {
  async fn get_attached(self, ups: &UpsName) -> Result<responses::AttachedDaemons, Error> {
    impl_pooled_call!(self.pool, get_attached, ups)
  }

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

  async fn list_cmd(self, ups: &UpsName) -> Result<responses::CmdList, Error> {
    impl_pooled_call!(self.pool, list_cmd, ups)
  }

  async fn list_enum(self, ups: &UpsName, var: &VarName) -> Result<responses::EnumList, Error> {
    impl_pooled_call!(self.pool, list_enum, ups, var)
  }

  async fn list_ranges(self, ups: &UpsName, var: &VarName) -> Result<responses::RangeList, Error> {
    impl_pooled_call!(self.pool, list_ranges, ups, var)
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

  async fn list_client(self, ups: &UpsName) -> Result<responses::ClientList, Error> {
    impl_pooled_call!(self.pool, list_client, ups)
  }
}
