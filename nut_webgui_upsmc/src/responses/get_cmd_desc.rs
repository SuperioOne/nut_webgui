use crate::CmdName;
use crate::internal::ReadOnlyStr;

pub struct GetCmdDesc {
  pub command: CmdName,
  pub desc: ReadOnlyStr,
}
