use nut_webgui_upsmc::ups_status::UpsStatus;

macro_rules! assert_status_text {
  (validate_str = true, $(($name:literal, $target:expr);)+) => {
    $(
        let status = nut_webgui_upsmc::ups_status::UpsStatus::new($name);

        assert_eq!(status, $target);
        assert_eq!(&status.to_string(), $name);
    )+
  };

  // NOTE: UpsStatus with multiple bits might have different order than input str.
  // Comparing `to_string()` does not makes sense.
  (validate_str = false ,$(($name:literal, $target:expr);)+) => {
    $(
        assert_eq!(nut_webgui_upsmc::ups_status::UpsStatus::new($name), $target);
    )+
  };
}

#[test]
fn single_status() {
  assert_status_text!(validate_str = true,
    ("ALARM",   UpsStatus::ALARM);
    ("BOOST",   UpsStatus::BOOST);
    ("BYPASS",  UpsStatus::BYPASS);
    ("CAL",     UpsStatus::CALIBRATING);
    ("CHRG",    UpsStatus::CHARGING);
    ("COMM",    UpsStatus::COMM);
    ("DISCHRG", UpsStatus::DISCHARGE);
    ("FSD",     UpsStatus::FORCED_SHUTDOWN);
    ("LB",      UpsStatus::LOW_BATTERY);
    ("NOCOMM",  UpsStatus::NOCOMM);
    ("OFF",     UpsStatus::OFFLINE);
    ("OL",      UpsStatus::ONLINE);
    ("OB",      UpsStatus::ON_BATTERY);
    ("OVER",    UpsStatus::OVERLOADED);
    ("RB",      UpsStatus::REPLACE_BATTERY);
    ("TEST",    UpsStatus::TEST);
    ("TICK",    UpsStatus::TICK);
    ("TOCK",    UpsStatus::TOCK);
    ("TRIM",    UpsStatus::TRIM);
  );
}

#[test]
fn with_multip_unspecified_status() {
  let status = UpsStatus::new("OB TEST OVERHEAT ECO MEGUSTA OL");
  assert_eq!(
    UpsStatus::ON_BATTERY | UpsStatus::TEST | UpsStatus::ONLINE,
    status
  )
}

#[test]
fn with_single_unspecified_status() {
  let status = UpsStatus::new("FANFAIL");

  assert_eq!(UpsStatus::default(), status)
}

#[test]
fn multiple_status() {
  assert_status_text!(validate_str = false,
    ("ALARM OL TEST",      UpsStatus::ALARM | UpsStatus::ONLINE | UpsStatus::TEST);
    ("OB DISCHRG",         UpsStatus::ON_BATTERY | UpsStatus::DISCHARGE);
    ("LB OB OVER",         UpsStatus::LOW_BATTERY | UpsStatus:: ON_BATTERY | UpsStatus::OVERLOADED);
    ("CAL TEST BOOST OL",  UpsStatus::CALIBRATING | UpsStatus::TEST | UpsStatus::BOOST | UpsStatus::ONLINE);
  );
}

#[test]
fn modify_status() {
  let status = UpsStatus::new("OB LB DISCHRG FSD");
  let new_status = status
    .set(UpsStatus::ONLINE)
    .unset(UpsStatus::ON_BATTERY | UpsStatus::DISCHARGE | UpsStatus::FORCED_SHUTDOWN);

  assert_eq!(new_status, UpsStatus::ONLINE | UpsStatus::LOW_BATTERY);
}
