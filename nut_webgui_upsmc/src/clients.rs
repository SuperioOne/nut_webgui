use crate::{
  CmdName, UpsName, VarName,
  errors::Error,
  responses::{
    ClientList, CmdDesc, CmdList, DaemonVer, EnumList, ProtVer, RangeList, RwList, UpsDesc,
    UpsList, UpsVar, UpsVarDesc, UpsVarList, UpsVarType,
  },
};
use core::future::Future;
mod client_auth;
mod client_base;
mod client_pool;

pub use client_auth::NutAuthClient;
pub use client_base::NutClient;
pub use client_pool::NutPoolClient;

pub trait AsyncNutClient {
  fn get_cmd_desc(
    self,
    ups: &UpsName,
    cmd: &CmdName,
  ) -> impl Future<Output = Result<CmdDesc, Error>>;

  fn get_protver(self) -> impl Future<Output = Result<ProtVer, Error>>;

  fn get_ups_desc(self, ups: &UpsName) -> impl Future<Output = Result<UpsDesc, Error>>;

  fn get_var(self, ups: &UpsName, var: &VarName) -> impl Future<Output = Result<UpsVar, Error>>;

  fn get_var_type(
    self,
    ups: &UpsName,
    var: &VarName,
  ) -> impl Future<Output = Result<UpsVarType, Error>>;

  fn get_var_desc(
    self,
    ups: &UpsName,
    var: &VarName,
  ) -> impl Future<Output = Result<UpsVarDesc, Error>>;

  fn get_ver(self) -> impl Future<Output = Result<DaemonVer, Error>>;

  fn list_client(self, ups: &UpsName) -> impl Future<Output = Result<ClientList, Error>>;

  fn list_cmd(self, ups: &UpsName) -> impl Future<Output = Result<CmdList, Error>>;

  fn list_enum(self, ups: &UpsName, var: &VarName)
  -> impl Future<Output = Result<EnumList, Error>>;

  fn list_range(
    self,
    ups: &UpsName,
    var: &VarName,
  ) -> impl Future<Output = Result<RangeList, Error>>;

  fn list_rw(self, ups: &UpsName) -> impl Future<Output = Result<RwList, Error>>;

  fn list_ups(self) -> impl Future<Output = Result<UpsList, Error>>;

  fn list_var(self, ups: &UpsName) -> impl Future<Output = Result<UpsVarList, Error>>;
}
