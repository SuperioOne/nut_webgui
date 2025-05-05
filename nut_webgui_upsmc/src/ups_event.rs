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
  BoostEnded,

  /// Bypass is on
  BypassOn,

  /// Bypass is off
  BypassOff,

  /// Calibrating
  Calibrating,

  /// Calibrating ended
  CalibrationEnded,

  /// Battery is charging
  Charging,

  /// Battery charging ended
  ChargeEnded,

  /// Battery is discharging
  Discharging,

  /// Battery discharge stoped
  DischargeEnded,

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
  TestEnded,

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
        UpsStatus::BOOST => events.insert(UpsEvent::BoostEnded),
        UpsStatus::BYPASS => events.insert(UpsEvent::BypassOff),
        UpsStatus::CALIBRATING => events.insert(UpsEvent::CalibrationEnded),
        UpsStatus::CHARGING => events.insert(UpsEvent::ChargeEnded),
        UpsStatus::DISCHARGE => events.insert(UpsEvent::DischargeEnded),
        UpsStatus::LOW_BATTERY => events.insert(UpsEvent::LowBatteryEnded),
        UpsStatus::OFFLINE => events.insert(UpsEvent::DeviceOn),
        UpsStatus::OVERLOADED => events.insert(UpsEvent::OverloadEnded),
        UpsStatus::REPLACE_BATTERY => events.insert(UpsEvent::ReplaceBatteryEnded),
        UpsStatus::TEST => events.insert(UpsEvent::TestEnded),
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

  pub fn contains(&self, event: UpsEvent) -> bool {
    self.events.contains(&event)
  }

  pub fn iter(&self) -> Iter<'_> {
    Iter {
      inner_iter: self.events.iter(),
    }
  }

  pub fn len(&self) -> usize {
    self.events.len()
  }

  pub fn is_empty(&self) -> bool {
    self.events.is_empty()
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
    let text = match self {
      UpsEvent::AlarmOn => "Alarm on",
      UpsEvent::AlarmOff => "Alarm off",
      UpsEvent::Boosting => "Boosting voltage",
      UpsEvent::BoostEnded => "Not boosting",
      UpsEvent::BypassOn => "Bypass on",
      UpsEvent::BypassOff => "Bypass off",
      UpsEvent::Calibrating => "Calibrating",
      UpsEvent::CalibrationEnded => "Not calibrating",
      UpsEvent::Charging => "Charging",
      UpsEvent::ChargeEnded => "Not charging",
      UpsEvent::Discharging => "Discharging",
      UpsEvent::DischargeEnded => "Not discharging",
      UpsEvent::FSD => "System shutdown",
      UpsEvent::LowBattery => "Low battery",
      UpsEvent::LowBatteryEnded => "Battery not low",
      UpsEvent::DeviceOff => "Ups turned off",
      UpsEvent::DeviceOn => "Ups not turned off",
      UpsEvent::Online => "Receiving power",
      UpsEvent::OnBattery => "Power lost",
      UpsEvent::Overloaded => "Ups overloaded",
      UpsEvent::OverloadEnded => "Ups not overloaded",
      UpsEvent::ReplaceBattery => "Replace battery",
      UpsEvent::ReplaceBatteryEnded => "Replace battery canceled",
      UpsEvent::Testing => "Test starts",
      UpsEvent::TestEnded => "Test finished",
      UpsEvent::Trimming => "Trimming voltage",
      UpsEvent::TrimEnded => "Not trimming",
      UpsEvent::NoCOMM => "Ups is dead",
      UpsEvent::COMM => "Ups communicating",
    };

    f.write_str(text)
  }
}
