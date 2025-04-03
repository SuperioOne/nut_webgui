use crate::internal::ReadOnlyStr;
use crate::{UpsName, VarName};

pub struct GetVarDesc {
  pub var_name: VarName,
  pub desc: ReadOnlyStr,
  pub ups: UpsName,
}
