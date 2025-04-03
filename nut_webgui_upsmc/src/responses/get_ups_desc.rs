use crate::UpsName;
use crate::internal::ReadOnlyStr;

pub struct GetUpsDesc {
  pub ups: UpsName,
  pub desc: ReadOnlyStr,
}
