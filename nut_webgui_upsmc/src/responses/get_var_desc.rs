use crate::VarName;
use crate::internal::ReadOnlyStr;

pub struct GetVarDesc {
  pub var_name: VarName,
  pub desc: ReadOnlyStr,
}
