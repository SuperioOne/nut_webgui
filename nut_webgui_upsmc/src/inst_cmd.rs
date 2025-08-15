use crate::CmdName;
#[cfg(feature = "serde")]
use serde::Serialize;
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone)]
pub struct InstCmd {
  pub id: CmdName,
  pub desc: String,
}
