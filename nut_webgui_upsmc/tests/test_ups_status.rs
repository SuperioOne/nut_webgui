use nut_webgui_upsmc::UpsStatus;

#[test]
fn from_str() {
  macro_rules! gen_status_tests {
  ($(($target:pat, $name:ident);)+) => {
    $(
        let status_text = stringify!($name);
        let status = match nut_webgui_upsmc::UpsStatus::try_from(status_text) {
          Ok(val @ $target) => val,
          Ok(val) => {
            return assert!(false, "Parsed status does not match to the expected target received={received:?} expected={expected:?}", received = val, expected = stringify!($target));
          },
          Err(_) => {
            return assert!(false, "Parser failed unexpectedly");
          },
        };

        assert_eq!(&status.to_string(), status_text);
    )+
  };
}

  gen_status_tests!(
  (UpsStatus::ALARM,           ALARM);
  (UpsStatus::BOOST,           BOOST);
  (UpsStatus::BYPASS,          BYPASS);
  (UpsStatus::CALIBRATING,     CAL);
  (UpsStatus::CHARGING,        CHRG);
  (UpsStatus::COMM,            COMM);
  (UpsStatus::DISCHARGE,       DISCHRG);
  (UpsStatus::FORCED_SHUTDOWN, FSD);
  (UpsStatus::LOW_BATTERY,     LB);
  (UpsStatus::NOCOMM,          NOCOMM);
  (UpsStatus::OFFLINE,         OFF);
  (UpsStatus::ONLINE,          OL);
  (UpsStatus::ON_BATTERY,      OB);
  (UpsStatus::OVERLOADED,      OVER);
  (UpsStatus::REPLACE_BATTERY, RB);
  (UpsStatus::TEST,            TEST);
  (UpsStatus::TICK,            TICK);
  (UpsStatus::TOCK,            TOCK);
  (UpsStatus::TRIM,            TRIM);
  );
}

#[test]
fn from_str_unspecified_status() {
  match UpsStatus::try_from("OB TEST WTF OL") {
    Ok(_) => assert!(false, "status parsing should've failed, but it passed?"),
    Err(_) => assert!(true),
  };
}

#[test]
fn multiple_status() {
  let target_status = UpsStatus::ON_BATTERY
    | UpsStatus::LOW_BATTERY
    | UpsStatus::DISCHARGE
    | UpsStatus::FORCED_SHUTDOWN;

  let status = UpsStatus::try_from("OB LB DISCHRG FSD").unwrap();

  assert!(status.has(UpsStatus::ON_BATTERY));
  assert!(status.has(UpsStatus::LOW_BATTERY));
  assert!(status.has(UpsStatus::DISCHARGE));
  assert!(status.has(UpsStatus::FORCED_SHUTDOWN));

  assert!(target_status.has(UpsStatus::ON_BATTERY));
  assert!(target_status.has(UpsStatus::LOW_BATTERY));
  assert!(target_status.has(UpsStatus::DISCHARGE));
  assert!(target_status.has(UpsStatus::FORCED_SHUTDOWN));

  assert_eq!(status, target_status)
}

#[test]
fn edit_status() {
  let status = UpsStatus::try_from("OB LB DISCHRG FSD").unwrap();
  let new_status = status
    .set(UpsStatus::ONLINE)
    .unset(UpsStatus::ON_BATTERY)
    .unset(UpsStatus::DISCHARGE)
    .unset(UpsStatus::FORCED_SHUTDOWN);

  assert_eq!(new_status, UpsStatus::ONLINE | UpsStatus::LOW_BATTERY);
}
