#![allow(unused_assignments, unused_mut)]

use nut_webgui_upsmc::{
  ups_event::{UpsEvent, UpsEvents},
  ups_status::UpsStatus,
};

macro_rules! test_event_stream {
  ($(($next_status:expr => [$($expected:expr),*]));+) => {
    let mut current = Default::default();

    $(
      let events = UpsEvents::new(current, $next_status);
      let mut expected_len = 0;
      current = $next_status;

      $(
        expected_len += 1;
        assert!(events.contains($expected), "Missing event={}", $expected);
      )*

      assert_eq!(events.len(), expected_len, "Event length does not match with the provided event count {:?}" ,events);
    )+
  };
}

#[test]
fn base_events() {
  test_event_stream!(
    (UpsStatus::ALARM                          => [UpsEvent::AlarmOn]);
    (UpsStatus::BOOST                          => [UpsEvent::AlarmOff, UpsEvent::Boosting]);
    (UpsStatus::BYPASS                         => [UpsEvent::BoostingEnded, UpsEvent::BypassOn]);
    (UpsStatus::CALIBRATING                    => [UpsEvent::BypassOff, UpsEvent::Calibrating]);
    (UpsStatus::CHARGING                       => [UpsEvent::Charging, UpsEvent::CalibrationCompleted]);
    (UpsStatus::COMM                           => [UpsEvent::ChargingEnded, UpsEvent::COMM]);
    (UpsStatus::DISCHARGE                      => [UpsEvent::Discharging]);
    (UpsStatus::LOW_BATTERY                    => [UpsEvent::DischargingEnded, UpsEvent::LowBattery]);
    (UpsStatus::ONLINE                         => [UpsEvent::LowBatteryEnded, UpsEvent::Online]);
    (UpsStatus::ON_BATTERY                     => [UpsEvent::OnBattery]);
    (UpsStatus::OVERLOADED | UpsStatus::ONLINE => [UpsEvent::Online, UpsEvent::Overloaded]);
    (UpsStatus::REPLACE_BATTERY                => [UpsEvent::OverloadEnded , UpsEvent::ReplaceBattery]);
    (UpsStatus::TEST                           => [UpsEvent::ReplaceBatteryEnded, UpsEvent::Testing]);
    (UpsStatus::TRIM                           => [UpsEvent::TestCompleted, UpsEvent::Trimming]);
    (UpsStatus::NOCOMM                         => [UpsEvent::TrimEnded, UpsEvent::NoCOMM]);
    (UpsStatus::FORCED_SHUTDOWN                => [UpsEvent::FSD]);
    (UpsStatus::OFFLINE                        => [UpsEvent::DeviceOff]);
    (UpsStatus::ONLINE                         => [UpsEvent::Online, UpsEvent::DeviceOn])
  );
}

#[test]
fn ups_status_simulation() {
  test_event_stream!(
    (UpsStatus::ONLINE | UpsStatus::ALARM                                  => [UpsEvent::Online, UpsEvent::AlarmOn]);
    (UpsStatus::ONLINE | UpsStatus::ALARM                                  => []);
    (UpsStatus::ONLINE                                                     => [UpsEvent::AlarmOff]);
    (UpsStatus::ONLINE                                                     => []);
    (UpsStatus::ON_BATTERY | UpsStatus::DISCHARGE                          => [UpsEvent::OnBattery, UpsEvent::Discharging]);
    (UpsStatus::ON_BATTERY | UpsStatus::DISCHARGE                          => []);
    (UpsStatus::ON_BATTERY | UpsStatus::DISCHARGE | UpsStatus::LOW_BATTERY => [UpsEvent::LowBattery]);
    (UpsStatus::ONLINE | UpsStatus::CHARGING | UpsStatus::LOW_BATTERY      => [UpsEvent::Online, UpsEvent::DischargingEnded, UpsEvent::Charging]);
    (UpsStatus::ONLINE | UpsStatus::CHARGING | UpsStatus::LOW_BATTERY      => []);
    (UpsStatus::ONLINE | UpsStatus::CHARGING                               => [UpsEvent::LowBatteryEnded]);
    (UpsStatus::ONLINE                                                     => [UpsEvent::ChargingEnded]);
    (UpsStatus::ONLINE | UpsStatus::TEST                                   => [UpsEvent::Testing]);
    (UpsStatus::ONLINE | UpsStatus::TEST                                   => []);
    (UpsStatus::ONLINE                                                     => [UpsEvent::TestCompleted]);
    (UpsStatus::ONLINE | UpsStatus::CALIBRATING                            => [UpsEvent::Calibrating]);
    (UpsStatus::ONLINE                                                     => [UpsEvent::CalibrationCompleted]);
    (UpsStatus::ONLINE                                                     => []);
    (UpsStatus::FORCED_SHUTDOWN                                            => [UpsEvent::FSD]);
    (UpsStatus::OFFLINE                                                    => [UpsEvent::DeviceOff])
  );
}
