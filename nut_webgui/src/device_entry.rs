use chrono::{DateTime, Utc};
use core::net::IpAddr;
use nut_webgui_upsmc::{
  CmdName, UpsName, Value, VarName, ups_status::UpsStatus, variables::UpsVariables,
};
use serde::{Serialize, ser::SerializeStruct};
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct DeviceEntry {
  pub attached: Vec<IpAddr>,
  pub commands: Vec<CmdName>,
  pub desc: Box<str>,
  pub last_modified: DateTime<Utc>,
  pub name: UpsName,
  pub rw_variables: HashMap<VarName, VarDetail>,
  pub status: UpsStatus,
  pub variables: UpsVariables,
}

#[derive(Debug)]
pub enum VarDetail {
  String { max_len: usize },
  Number,
  Enum { options: Vec<Value> },
  Range { min: Value, max: Value },
}

impl Serialize for VarDetail {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    match self {
      VarDetail::String { max_len } => {
        let mut ser = serializer.serialize_struct("VarDetail", 2)?;
        ser.serialize_field("type", "string")?;
        ser.serialize_field("max_len", max_len)?;
        ser.end()
      }
      VarDetail::Number => {
        let mut ser = serializer.serialize_struct("VarDetail", 1)?;
        ser.serialize_field("type", "number")?;
        ser.end()
      }
      VarDetail::Enum { options } => {
        let mut ser = serializer.serialize_struct("VarDetail", 2)?;
        ser.serialize_field("type", "enum")?;
        ser.serialize_field("options", options)?;
        ser.end()
      }
      VarDetail::Range { min, max } => {
        let mut ser = serializer.serialize_struct("VarDetail", 3)?;
        ser.serialize_field("type", "range")?;
        ser.serialize_field("min", min)?;
        ser.serialize_field("max", max)?;
        ser.end()
      }
    }
  }
}
