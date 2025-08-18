use nut_webgui_upsmc::clients::AsyncNutClient;
use nut_webgui_upsmc::{CmdName, UpsName, Value, VarName, VarType};

#[tokio::test]
async fn cmd_desc() {
  const CMD_DESC: &[u8] = b"CMDDESC bx1600mi beeper.on \"Obsolete (use beeper.enable)\"\n";
  let ups = UpsName::new_unchecked("bx1600mi");
  let cmd = CmdName::new_unchecked("beeper.on");

  let stream = tokio_test::io::Builder::new()
    .write(format!("GET CMDDESC {ups} {cmd}\n", ups = &ups, cmd = &cmd).as_bytes())
    .read(CMD_DESC)
    .build();

  let mut client = nut_webgui_upsmc::clients::NutClient::from(stream);
  let cmd_desc = client.get_cmd_desc(&ups, &cmd).await.unwrap();

  assert_eq!(cmd_desc.cmd, cmd);
  assert_eq!(cmd_desc.ups_name, ups);
  assert_eq!(cmd_desc.desc.as_ref(), "Obsolete (use beeper.enable)");
}

#[tokio::test]
async fn var_desc() {
  const VAR_DESC: &[u8] = b"DESC bx1600mi ups.beeper.status \"UPS beeper status\"\n";
  let ups = UpsName::new_unchecked("bx1600mi");

  let stream = tokio_test::io::Builder::new()
    .write(
      format!(
        "GET DESC {ups} {var}\n",
        ups = &ups,
        var = VarName::UPS_BEEPER_STATUS
      )
      .as_bytes(),
    )
    .read(VAR_DESC)
    .build();

  let mut client = nut_webgui_upsmc::clients::NutClient::from(stream);
  let var_desc = client
    .get_var_desc(&ups, VarName::UPS_BEEPER_STATUS)
    .await
    .unwrap();

  assert_eq!(var_desc.name, VarName::UPS_BEEPER_STATUS);
  assert_eq!(var_desc.ups_name, ups);
  assert_eq!(var_desc.desc.as_ref(), "UPS beeper status");
}

#[tokio::test]
async fn var_type() {
  const VAR_TYPE: &[u8] = b"TYPE bx1600mi ups.custom RW ENUM STRING:64 NUMBER RANGE\n";
  let ups = UpsName::new_unchecked("bx1600mi");
  let var_name = VarName::new_unchecked("ups.custom");

  let stream = tokio_test::io::Builder::new()
    .write(format!("GET TYPE {ups} {var}\n", ups = &ups, var = &var_name).as_bytes())
    .read(VAR_TYPE)
    .build();

  let mut client = nut_webgui_upsmc::clients::NutClient::from(stream);
  let var_desc = client.get_var_type(&ups, &var_name).await.unwrap();

  assert_eq!(var_desc.name, var_name);
  assert_eq!(var_desc.ups_name, ups);
  assert!(var_desc.var_types.contains(&VarType::Enum));
  assert!(var_desc.var_types.contains(&VarType::ReadWrite));
  assert!(
    var_desc
      .var_types
      .contains(&VarType::String { max_len: 64 })
  );
  assert!(var_desc.var_types.contains(&VarType::Range));
  assert!(var_desc.var_types.contains(&VarType::Number));
}

#[tokio::test]
async fn list_ups() {
  const LIST_UPS: &[u8] = b"BEGIN LIST UPS
UPS bx1600mi \"APC Back-UPS\\\\ BX1600MI\"
END LIST UPS
";

  let stream = tokio_test::io::Builder::new()
    .write(b"LIST UPS\n")
    .read(LIST_UPS)
    .build();

  let mut client = nut_webgui_upsmc::clients::NutClient::from(stream);
  let ups_list = client.list_ups().await.unwrap();

  assert_eq!(ups_list.devices.len(), 1);

  let device = ups_list.devices.get(0).unwrap();

  assert_eq!(device.ups_name.to_string(), "bx1600mi");
  assert_eq!(device.desc.as_ref(), "APC Back-UPS\\ BX1600MI");
}

#[tokio::test]
async fn get_var() {
  const GET_VAR: &[u8] = b"VAR bx1600mi ups.beeper.status \"disabled\"\n";

  let ups = UpsName::new_unchecked("bx1600mi");

  let stream = tokio_test::io::Builder::new()
    .write(
      format!(
        "GET VAR {ups} {var}\n",
        ups = &ups,
        var = VarName::UPS_BEEPER_STATUS
      )
      .as_bytes(),
    )
    .read(GET_VAR)
    .build();

  let mut client = nut_webgui_upsmc::clients::NutClient::from(stream);
  let var = client
    .get_var(&ups, VarName::UPS_BEEPER_STATUS)
    .await
    .unwrap();

  assert_eq!(var.ups_name, ups);
  assert_eq!(var.name, VarName::UPS_BEEPER_STATUS);
  assert_eq!(var.value, "disabled");
}

#[tokio::test]
async fn list_client() {
  const LIST_CLIENT: &[u8] = b"BEGIN LIST CLIENT bx1600mi
CLIENT bx1600mi 127.0.0.1
END LIST CLIENT bx1600mi
";

  let ups = UpsName::new_unchecked("bx1600mi");

  let stream = tokio_test::io::Builder::new()
    .write(format!("LIST CLIENT {ups}\n", ups = &ups,).as_bytes())
    .read(LIST_CLIENT)
    .build();

  let mut client = nut_webgui_upsmc::clients::NutClient::from(stream);
  let client_list = client.list_client(&ups).await.unwrap();

  assert_eq!(client_list.ups_name, ups);
  assert_eq!(client_list.ips.len(), 1);

  let ip = client_list.ips.get(0).unwrap();

  assert!(ip.is_ipv4());
  assert!(ip.is_loopback());
}

#[tokio::test]
async fn list_cmd() {
  macro_rules! gen_cmd_test {
      ($($cmd:literal),+) => {
        let mut cmd_len = 0;

        $(
          cmd_len += { _ = $cmd; 1 };
        )+

        let ups = nut_webgui_upsmc::UpsName::new_unchecked("bx1600mi");
        const INPUT : &str = concat!(
          "BEGIN LIST CMD bx1600mi\n",
          $("CMD bx1600mi ", $cmd, "\n",)+
          "END LIST CMD bx1600mi\n",
        );

        let stream = tokio_test::io::Builder::new()
          .write(format!("LIST CMD {ups}\n", ups = &ups,).as_bytes())
          .read(INPUT.as_bytes())
          .build();

        let mut client = nut_webgui_upsmc::clients::NutClient::from(stream);
        let cmd_list = client.list_cmd(&ups).await.unwrap();

        assert_eq!(cmd_list.ups_name, ups);
        assert_eq!(cmd_list.cmds.len(), cmd_len);

        let mut iter = cmd_list.cmds.iter();

        $(
          assert_eq!(iter.next().unwrap(), $cmd);
        )+

        assert_eq!(iter.next(), None)
      };
  }

  gen_cmd_test!(
    "beeper.disable",
    "beeper.enable",
    "beeper.mute",
    "beeper.off",
    "beeper.on",
    "driver.killpower",
    "driver.reload",
    "driver.reload-or-error",
    "driver.reload-or-exit",
    "load.off",
    "load.off.delay",
    "shutdown.reboot",
    "shutdown.stop",
    "test.battery.start.deep",
    "test.battery.start.quick",
    "test.battery.stop"
  );
}

#[tokio::test]
async fn list_var() {
  macro_rules! gen_var_test {
      ($(($name:literal, $value:literal);)+) => {
        let mut var_len = 0;

        $(
          var_len += { _ = $name; 1 };
        )+

        let ups = nut_webgui_upsmc::UpsName::new_unchecked("bx1600mi");
        const INPUT : &str = concat!(
          "BEGIN LIST VAR bx1600mi\n",
          $("VAR bx1600mi ",$name, " \"",  $value, "\"\n",)+
          "END LIST VAR bx1600mi\n",
        );

        let stream = tokio_test::io::Builder::new()
          .write(format!("LIST VAR {ups}\n", ups = &ups,).as_bytes())
          .read(INPUT.as_bytes())
          .build();

        let mut client = nut_webgui_upsmc::clients::NutClient::from(stream);
        let var_list = client.list_var(&ups).await.unwrap();

        assert_eq!(var_list.ups_name, ups);
        assert_eq!(var_list.variables.len(), var_len);

        $(
          match var_list.variables.get(VarName::new_unchecked($name)) {
            Some(value) => assert_eq!(value, $value),
            None => assert!(false)
          }
        )+
      };
  }

  gen_var_test!(
    ("battery.charge", 100);
    ("battery.charge.low", 30);
    ("battery.mfr.date", "2001/01/01");
    ("battery.runtime", 1160);
    ("battery.runtime.low", 240);
    ("battery.type", "PbAc");
    ("battery.voltage", 27.3);
    ("battery.voltage.nominal", 24.0);
    ("device.mfr", "American Power Conversion");
    ("device.model", "Back-UPS BX1600MI");
    ("device.serial", "9b000000000a");
    ("device.type", "ups");
    ("driver.debug", 0);
    ("driver.flag.allow_killpower",  0);
    ("driver.name", "usbhid-ups");
    ("driver.parameter.pollfreq", 30);
    ("driver.parameter.pollinterval", 1);
    ("driver.parameter.port", "auto");
    ("driver.parameter.productid", "0002");
    ("driver.parameter.serial", "9b000000000a");
    ("driver.parameter.synchronous", "auto");
    ("driver.parameter.vendorid", "051D");
    ("driver.state", "updateinfo");
    ("driver.version", "2.8.1");
    ("driver.version.data", "APC HID 0.100");
    ("driver.version.internal", 0.52);
    ("driver.version.usb", "libusb-1.0.27 (API: 0x100010a)");
    ("input.sensitivity", "medium");
    ("input.transfer.high", 295);
    ("input.transfer.low", 145);
    ("input.voltage", 232.0);
    ("input.voltage.nominal", 230);
    ("ups.beeper.status", "disabled");
    ("ups.delay.shutdown", 20);
    ("ups.firmware", "378600G -302202G ");
    ("ups.load", 27);
    ("ups.mfr", "American Power Conversion");
    ("ups.mfr.date", "2023/02/25");
    ("ups.model", "Back-UPS BX1600MI");
    ("ups.productid", "0002");
    ("ups.realpower.nominal", 900);
    ("ups.serial", "9b000000000a");
    ("ups.status", "OL");
    ("ups.test.result",  "Done and passed");
    ("ups.timer.reboot", 0);
    ("ups.timer.shutdown", -1);
    ("ups.vendorid", "051d");
    ("custom", 0.1);
  );
}

#[tokio::test]
async fn list_range() {
  const LIST_RANGE: &[u8] = b"BEGIN LIST RANGE bx1600mi input.transfer.low
RANGE bx1600mi input.transfer.low \"90\" \"105\"
END LIST RANGE bx1600mi input.transfer.low
";

  let ups = UpsName::new_unchecked("bx1600mi");

  let stream = tokio_test::io::Builder::new()
    .write(
      format!(
        "LIST RANGE {ups} {var}\n",
        ups = &ups,
        var = VarName::INPUT_TRANSFER_LOW
      )
      .as_bytes(),
    )
    .read(LIST_RANGE)
    .build();

  let mut client = nut_webgui_upsmc::clients::NutClient::from(stream);
  let range_list = client
    .list_range(&ups, VarName::INPUT_TRANSFER_LOW)
    .await
    .unwrap();

  assert_eq!(range_list.ups_name, ups);
  assert_eq!(range_list.name, VarName::INPUT_TRANSFER_LOW);

  let range = range_list.ranges.get(0).unwrap();

  assert_eq!(range.0, 90);
  assert_eq!(range.1, 105);
}

#[tokio::test]
async fn list_enum() {
  const LIST_ENUM: &[u8] = b"BEGIN LIST ENUM bx1600mi input.transfer.low
ENUM bx1600mi input.transfer.low \"103\"
ENUM bx1600mi input.transfer.low \"100\"
END LIST ENUM bx1600mi input.transfer.low
";

  let ups = UpsName::new_unchecked("bx1600mi");

  let stream = tokio_test::io::Builder::new()
    .write(
      format!(
        "LIST ENUM {ups} {var}\n",
        ups = &ups,
        var = VarName::INPUT_TRANSFER_LOW
      )
      .as_bytes(),
    )
    .read(LIST_ENUM)
    .build();

  let mut client = nut_webgui_upsmc::clients::NutClient::from(stream);
  let enum_list = client
    .list_enum(&ups, VarName::INPUT_TRANSFER_LOW)
    .await
    .unwrap();

  assert_eq!(enum_list.ups_name, ups);
  assert_eq!(enum_list.name, VarName::INPUT_TRANSFER_LOW);

  let first = enum_list.values.get(0).unwrap();
  let second = enum_list.values.get(1).unwrap();

  assert_eq!(enum_list.values.len(), 2);
  assert_eq!(*first, 103);
  assert_eq!(*second, 100);
}

#[tokio::test]
async fn list_rw() {
  macro_rules! gen_rw_test {
      ($(($name:literal, $value:literal);)+) => {
        let mut rw_len = 0;

        $(
          rw_len += { _ = $name; 1 };
        )+

        let ups = nut_webgui_upsmc::UpsName::new_unchecked("bx1600mi");

        const INPUT : &str = concat!(
          "BEGIN LIST RW bx1600mi\n",
          $("RW bx1600mi ", $name, " \"", $value, "\"\n",)+
          "END LIST RW bx1600mi\n",
        );

        let stream = tokio_test::io::Builder::new()
          .write(format!("LIST RW {ups}\n", ups = &ups).as_bytes())
          .read(INPUT.as_bytes())
          .build();

        let mut client = nut_webgui_upsmc::clients::NutClient::from(stream);
        let rw_list = client.list_rw(&ups).await.unwrap();

        assert_eq!(rw_list.ups_name, ups);
        assert_eq!(rw_list.variables.len(), rw_len);

        $(
          match rw_list.variables.get(&VarName::new_unchecked($name)) {
            Some(value) => assert_eq!(*value, $value),
            None => assert!(false)
          }
        )+
      };
  }

  gen_rw_test!(
    ("battery.charge.low", 30);
    ("battery.mfr.date", "2001/01/01");
    ("battery.runtime.low", 240);
    ("driver.debug", 0);
    ("driver.flag.allow_killpower", 0);
    ("input.sensitivity", "medium");
    ("input.transfer.high", 295);
    ("input.transfer.low", 145);
    ("ups.delay.shutdown", 20);
  );
}

#[tokio::test]
async fn get_versions() {
  const PROT_VER: &[u8] = b"1.3\n";
  const DAEMON_VER: &[u8] = b"NETWORK UPS Tools upsd 2.8.1 - https://www.networkupstools.org/\n";

  let stream = tokio_test::io::Builder::new()
    .write(b"VER\n")
    .read(DAEMON_VER)
    .write(b"NETVER\n")
    .read(PROT_VER)
    .build();

  let mut client = nut_webgui_upsmc::clients::NutClient::from(stream);
  let ver = client.get_ver().await.unwrap();
  let protver = client.get_protver().await.unwrap();

  assert_eq!(
    ver.value,
    "NETWORK UPS Tools upsd 2.8.1 - https://www.networkupstools.org/"
  );
  assert_eq!(protver.value, "1.3");
}

#[tokio::test]
async fn instcmd() {
  let ups = nut_webgui_upsmc::UpsName::new_unchecked("bx1600mi");
  let stream = tokio_test::io::Builder::new()
    .write(b"USERNAME user\n")
    .read(b"OK\n")
    .write(b"PASSWORD password\n")
    .read(b"OK\n")
    .write(b"INSTCMD bx1600mi beeper.on\n")
    .read(b"OK\n")
    .build();

  let mut client = nut_webgui_upsmc::clients::NutClient::from(stream)
    .authenticate("user", "password")
    .await
    .unwrap();

  client
    .instcmd(ups, CmdName::new_unchecked("beeper.on"))
    .await
    .unwrap();
}

#[tokio::test]
async fn fsd() {
  let ups = nut_webgui_upsmc::UpsName::new_unchecked("home:bx1600mi@localhost:4242");
  let stream = tokio_test::io::Builder::new()
    .write(b"USERNAME user\n")
    .read(b"OK\n")
    .write(b"PASSWORD password\n")
    .read(b"OK\n")
    .write(b"FSD home:bx1600mi@localhost:4242\n")
    .read(b"OK FSD-SET\n")
    .build();

  let mut client = nut_webgui_upsmc::clients::NutClient::from(stream)
    .authenticate("user", "password")
    .await
    .unwrap();

  client.fsd(ups).await.unwrap();
}

#[tokio::test]
async fn set_var() {
  let ups = nut_webgui_upsmc::UpsName::new_unchecked("bx1600mi");
  let stream = tokio_test::io::Builder::new()
    .write(b"USERNAME user\n")
    .read(b"OK\n")
    .write(b"PASSWORD password\n")
    .read(b"OK\n")
    .write(
      format!(
        "SET VAR {ups} {var} \"{value}\"\n",
        ups = &ups,
        var = VarName::BATTERY_RUNTIME_LOW,
        value = 32
      )
      .as_bytes(),
    )
    .read(b"OK\n")
    .build();

  let mut client = nut_webgui_upsmc::clients::NutClient::from(stream)
    .authenticate("user", "password")
    .await
    .unwrap();

  client
    .set_var(ups, VarName::BATTERY_RUNTIME_LOW, Value::from(32))
    .await
    .unwrap();
}

#[tokio::test]
async fn attach_ups_and_detach() {
  let ups = nut_webgui_upsmc::UpsName::new_unchecked("bx1600mi");
  let stream = tokio_test::io::Builder::new()
    .write(b"USERNAME user\n")
    .read(b"OK\n")
    .write(b"PASSWORD password\n")
    .read(b"OK\n")
    .write(format!("LOGIN {ups}\n", ups = &ups,).as_bytes())
    .read(b"OK\n")
    .write(format!("LOGOUT\n").as_bytes())
    .read(b"OK Goodbye\n")
    .build();

  let mut client = nut_webgui_upsmc::clients::NutClient::from(stream)
    .authenticate("user", "password")
    .await
    .unwrap();

  client.attach(&ups).await.unwrap();
  client.detach().await.unwrap();
}
