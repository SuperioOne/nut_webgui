use crate::internal::ReadOnlyStr;

pub struct Ok;
pub struct OkFsd;
pub struct OkTls;
pub struct OkDetach;

pub struct ProtVer {
  pub ver: ReadOnlyStr,
}

pub struct DaemonVer {
  pub ver: ReadOnlyStr,
}
