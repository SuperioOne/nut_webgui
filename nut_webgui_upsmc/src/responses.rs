mod get_cmd_desc;
mod get_ups_desc;
mod get_var;
mod get_var_desc;
mod list_cmd;
mod list_enum;
mod list_range;
mod list_rw;
mod list_ups;
mod list_var;
mod misc;

trait DeserializeResponse: Sized {
  type Error;

  fn deserialize_from(bytes: &[u8]) -> Result<Self, Self::Error>;
}

pub use get_cmd_desc::*;
pub use get_ups_desc::*;
pub use get_var::*;
pub use get_var_desc::*;
pub use list_cmd::*;
pub use list_enum::*;
pub use list_range::*;
pub use list_rw::*;
pub use list_ups::*;
pub use list_var::*;
pub use misc::*;
