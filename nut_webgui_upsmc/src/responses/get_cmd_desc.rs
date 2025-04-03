use crate::internal::ReadOnlyStr;
use crate::{CmdName, UpsName};

pub struct GetCmdDesc {
  pub command: CmdName,
  pub desc: ReadOnlyStr,
  pub ups: UpsName,
}
