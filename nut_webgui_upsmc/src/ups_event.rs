use crate::ups_status::UpsStatus;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UpsEvent {
  /// Alarm is on
  AlarmOn,

  /// Alarm is off
  AlarmOff,

  /// Boosting voltage
  Boosting,

  /// Voltage boosting ended
  BoostingEnded,

  /// Bypass is on
  BypassOn,

  /// Bypass is off
  BypassOff,

  /// Calibrating
  Calibrating,

  /// Calibrating ended
  CalibrationCompleted,

  /// Battery is charging
  Charging,

  /// Battery charging ended
  ChargingEnded,

  /// Battery is discharging
  Discharging,

  /// Battery discharge stoped
  DischargingEnded,

  /// Forced shutdown initiated (Execute Order 66)
  FSD,

  /// Battery is low (LOWBATT)
  LowBattery,

  /// Battery level normal
  LowBatteryEnded,

  /// Device is turned off
  DeviceOff,

  /// Device is not turned off
  DeviceOn,

  /// Receiving power from wall (ONLINE)
  Online,

  /// Device is on battery (ONBATT)
  OnBattery,

  /// Ups is overloaded
  Overloaded,

  /// Ups is no longer overloaded
  OverloadEnded,

  /// Battery requires replacement (REPLBATT)
  ReplaceBattery,

  /// Either battery replaced or replacement cancelled
  ReplaceBatteryEnded,

  /// Test started
  Testing,

  /// Test completed
  TestCompleted,

  /// Trimming voltage
  Trimming,

  /// Voltage trimming ended
  TrimEnded,

  /// Ups is dead (COMMBAD, NOCOMM)
  NoCOMM,

  /// Ups communicating
  COMM,
}

#[derive(Debug, Clone)]
pub struct UpsEvents {
  events: HashSet<UpsEvent>,
}

impl UpsEvents {
  pub fn new(old_status: UpsStatus, new_status: UpsStatus) -> Self {
    let mut events: HashSet<UpsEvent> = HashSet::new();
    let removed = old_status.unset(new_status);

    for status in removed {
      _ = match status {
        UpsStatus::ALARM => events.insert(UpsEvent::AlarmOff),
        UpsStatus::BOOST => events.insert(UpsEvent::BoostingEnded),
        UpsStatus::BYPASS => events.insert(UpsEvent::BypassOff),
        UpsStatus::CALIBRATING => events.insert(UpsEvent::CalibrationCompleted),
        UpsStatus::CHARGING => events.insert(UpsEvent::ChargingEnded),
        UpsStatus::DISCHARGE => events.insert(UpsEvent::DischargingEnded),
        UpsStatus::LOW_BATTERY => events.insert(UpsEvent::LowBatteryEnded),
        UpsStatus::OFFLINE => events.insert(UpsEvent::DeviceOn),
        UpsStatus::OVERLOADED => events.insert(UpsEvent::OverloadEnded),
        UpsStatus::REPLACE_BATTERY => events.insert(UpsEvent::ReplaceBatteryEnded),
        UpsStatus::TEST => events.insert(UpsEvent::TestCompleted),
        UpsStatus::TRIM => events.insert(UpsEvent::TrimEnded),
        _ => false,
      };
    }

    let added = new_status.unset(old_status);

    for status in added {
      _ = match status {
        UpsStatus::ALARM => events.insert(UpsEvent::AlarmOn),
        UpsStatus::BOOST => events.insert(UpsEvent::Boosting),
        UpsStatus::BYPASS => events.insert(UpsEvent::BypassOn),
        UpsStatus::CALIBRATING => events.insert(UpsEvent::Calibrating),
        UpsStatus::CHARGING => events.insert(UpsEvent::Charging),
        UpsStatus::COMM => events.insert(UpsEvent::COMM),
        UpsStatus::DISCHARGE => events.insert(UpsEvent::Discharging),
        UpsStatus::FORCED_SHUTDOWN => events.insert(UpsEvent::FSD),
        UpsStatus::LOW_BATTERY => events.insert(UpsEvent::LowBattery),
        UpsStatus::NOCOMM => events.insert(UpsEvent::NoCOMM),
        UpsStatus::OFFLINE => events.insert(UpsEvent::DeviceOff),
        UpsStatus::ONLINE => events.insert(UpsEvent::Online),
        UpsStatus::ON_BATTERY => events.insert(UpsEvent::OnBattery),
        UpsStatus::OVERLOADED => events.insert(UpsEvent::Overloaded),
        UpsStatus::REPLACE_BATTERY => events.insert(UpsEvent::ReplaceBattery),
        UpsStatus::TEST => events.insert(UpsEvent::Testing),
        UpsStatus::TRIM => events.insert(UpsEvent::Trimming),
        _ => false,
      }
    }

    Self { events }
  }

  #[inline]
  pub fn contains(&self, event: UpsEvent) -> bool {
    self.events.contains(&event)
  }

  pub fn iter(&self) -> Iter<'_> {
    Iter {
      inner_iter: self.events.iter(),
    }
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.events.len()
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.events.is_empty()
  }
}

impl UpsEvent {
  pub const fn as_str(&self) -> &'static str {
    match self {
      UpsEvent::AlarmOn => "AlarmOn",
      UpsEvent::AlarmOff => "AlarmOff",
      UpsEvent::Boosting => "Boosting",
      UpsEvent::BoostingEnded => "BoostingEnded",
      UpsEvent::BypassOn => "BypassOn",
      UpsEvent::BypassOff => "BypassOff",
      UpsEvent::Calibrating => "Calibrating",
      UpsEvent::CalibrationCompleted => "CalibrationCompleted",
      UpsEvent::Charging => "Charging",
      UpsEvent::ChargingEnded => "ChargingEnded",
      UpsEvent::Discharging => "Discharging",
      UpsEvent::DischargingEnded => "DischargingEnded",
      UpsEvent::FSD => "FSD",
      UpsEvent::LowBattery => "LowBattery",
      UpsEvent::LowBatteryEnded => "LowBatteryEnded",
      UpsEvent::DeviceOff => "DeviceOff",
      UpsEvent::DeviceOn => "DeviceOn",
      UpsEvent::Online => "Online",
      UpsEvent::OnBattery => "OnBattery",
      UpsEvent::Overloaded => "Overloaded",
      UpsEvent::OverloadEnded => "OverloadEnded",
      UpsEvent::ReplaceBattery => "ReplaceBattery",
      UpsEvent::ReplaceBatteryEnded => "ReplaceBatteryEnded",
      UpsEvent::Testing => "Testing",
      UpsEvent::TestCompleted => "TestCompleted",
      UpsEvent::Trimming => "Trimming",
      UpsEvent::TrimEnded => "TrimEnded",
      UpsEvent::NoCOMM => "NoCOMM",
      UpsEvent::COMM => "COMM",
    }
  }
}

pub struct Iter<'a> {
  inner_iter: std::collections::hash_set::Iter<'a, UpsEvent>,
}

impl<'a> Iterator for Iter<'a> {
  type Item = &'a UpsEvent;

  fn next(&mut self) -> Option<Self::Item> {
    self.inner_iter.next()
  }
}

impl<'a> IntoIterator for &'a UpsEvents {
  type Item = &'a UpsEvent;

  type IntoIter = Iter<'a>;

  fn into_iter(self) -> Self::IntoIter {
    Iter {
      inner_iter: self.events.iter(),
    }
  }
}

impl std::fmt::Display for UpsEvent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.as_str())
  }
}

#[cfg(feature = "serde")]
mod serde {
  use super::{UpsEvent, UpsEvents};
  use serde::ser::SerializeSeq;

  impl serde::Serialize for UpsEvents {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: serde::Serializer,
    {
      let mut seq_serializer = serializer.serialize_seq(Some(self.events.len()))?;

      for event in self.events.iter() {
        seq_serializer.serialize_element(event)?;
      }

      seq_serializer.end()
    }
  }

  impl serde::Serialize for UpsEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: serde::Serializer,
    {
      serializer.serialize_str(self.as_str())
    }
  }
}
