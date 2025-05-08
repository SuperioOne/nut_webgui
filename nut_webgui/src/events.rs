use nut_webgui_upsmc::{UpsName, Value, variables::UpsVariables};

use crate::state::DeviceEntry;

#[derive(Debug, Clone)]
pub enum SystemEvents {
  /// Updates all device variables.
  FullUpdate {
    data: Vec<(UpsName, UpsVariables)>,
  },

  /// Marks daemon as dead
  DaemonIsDead,

  NewDevice {
    device: DeviceEntry,
  },

  RemovedDevice {
    name: UpsName,
  },

  StatusUpdates {
    data: Vec<(UpsName, Value)>,
  },
}
