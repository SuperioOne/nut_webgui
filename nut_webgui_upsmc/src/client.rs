use crate::{
  CmdName, UpsName, VarName,
  error::Error,
  response::{
    ClientList, CmdDesc, CmdList, DaemonVer, EnumList, ProtVer, RangeList, RwList, UpsDesc,
    UpsList, UpsVar, UpsVarDesc, UpsVarList, UpsVarType,
  },
};
use core::borrow::Borrow;
mod client_auth;
mod client_base;
mod client_pool;

#[cfg(feature = "rustls")]
mod client_tls;

pub use client_auth::NutAuthClient;
pub use client_base::NutClient;
pub use client_pool::{ClientStream, NutPoolClient, NutPoolClientBuilder};

pub trait AsyncNutClient {
  fn get_cmd_desc<N, C>(self, ups: N, cmd: C) -> impl Future<Output = Result<CmdDesc, Error>>
  where
    N: Borrow<UpsName>,
    C: Borrow<CmdName>;

  fn get_protver(self) -> impl Future<Output = Result<ProtVer, Error>>;

  fn get_ups_desc<N>(self, ups: N) -> impl Future<Output = Result<UpsDesc, Error>>
  where
    N: Borrow<UpsName>;

  fn get_var<N, V>(self, ups: N, var: V) -> impl Future<Output = Result<UpsVar, Error>>
  where
    N: Borrow<UpsName>,
    V: Borrow<VarName>;

  fn get_var_type<N, V>(self, ups: N, var: V) -> impl Future<Output = Result<UpsVarType, Error>>
  where
    N: Borrow<UpsName>,
    V: Borrow<VarName>;

  fn get_var_desc<N, V>(self, ups: N, var: V) -> impl Future<Output = Result<UpsVarDesc, Error>>
  where
    N: Borrow<UpsName>,
    V: Borrow<VarName>;

  fn get_ver(self) -> impl Future<Output = Result<DaemonVer, Error>>;

  fn list_client<N>(self, ups: N) -> impl Future<Output = Result<ClientList, Error>>
  where
    N: Borrow<UpsName>;

  fn list_cmd<N>(self, ups: N) -> impl Future<Output = Result<CmdList, Error>>
  where
    N: Borrow<UpsName>;

  fn list_enum<N, V>(self, ups: N, var: V) -> impl Future<Output = Result<EnumList, Error>>
  where
    N: Borrow<UpsName>,
    V: Borrow<VarName>;

  fn list_range<N, V>(self, ups: N, var: V) -> impl Future<Output = Result<RangeList, Error>>
  where
    N: Borrow<UpsName>,
    V: Borrow<VarName>;

  fn list_rw<N>(self, ups: N) -> impl Future<Output = Result<RwList, Error>>
  where
    N: Borrow<UpsName>;

  fn list_ups(self) -> impl Future<Output = Result<UpsList, Error>>;

  fn list_var<N>(self, ups: N) -> impl Future<Output = Result<UpsVarList, Error>>
  where
    N: Borrow<UpsName>;
}
