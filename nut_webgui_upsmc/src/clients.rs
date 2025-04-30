use crate::{
  CmdName, UpsName, VarName,
  errors::Error,
  responses::{
    AttachedDaemons, CmdDesc, CmdList, DaemonVer, EnumList, ProtVer, RangeList, RwList, UpsDesc,
    UpsList, UpsVar, UpsVarDesc, UpsVarList,
  },
};
use core::future::Future;
mod client_auth;
mod client_pool;
mod client_tcp;

pub use client_auth::NutAuthClient;
pub use client_pool::NutPoolClient;
pub use client_tcp::NutTcpClient;

pub trait NutClient {
  fn get_ver(self) -> impl Future<Output = Result<DaemonVer, Error>>;
  fn get_protver(self) -> impl Future<Output = Result<ProtVer, Error>>;

  fn get_attached(self, ups: &UpsName) -> impl Future<Output = Result<AttachedDaemons, Error>>;

  fn get_cmd_desc(
    self,
    ups: &UpsName,
    cmd: &CmdName,
  ) -> impl Future<Output = Result<CmdDesc, Error>>;

  fn get_cmd_list(self, ups_name: &UpsName) -> impl Future<Output = Result<CmdList, Error>>;

  fn get_enum_list(
    self,
    ups: &UpsName,
    var: &VarName,
  ) -> impl Future<Output = Result<EnumList, Error>>;

  fn get_range_list(
    self,
    ups: &UpsName,
    var: &VarName,
  ) -> impl Future<Output = Result<RangeList, Error>>;

  fn get_rw_list(self, ups: &UpsName) -> impl Future<Output = Result<RwList, Error>>;

  fn get_ups_desc(self, ups: &UpsName) -> impl Future<Output = Result<UpsDesc, Error>>;

  fn get_ups_list(self) -> impl Future<Output = Result<UpsList, Error>>;

  fn get_var(self, ups: &UpsName, var: &VarName) -> impl Future<Output = Result<UpsVar, Error>>;

  fn get_var_desc(
    self,
    ups: &UpsName,
    var: &VarName,
  ) -> impl Future<Output = Result<UpsVarDesc, Error>>;

  fn get_var_list(self, ups_name: &UpsName) -> impl Future<Output = Result<UpsVarList, Error>>;
}
