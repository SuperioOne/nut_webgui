use nut_webgui_upsmc::client::AsyncNutClient;
use nut_webgui_upsmc::error::ProtocolError;

macro_rules! gen_prot_err_tests {
  ($(($test_name:ident, $expected:pat, $error:literal );)+) => {
    $(
      #[tokio::test]
      async fn $test_name() {
        let stream = tokio_test::io::Builder::new()
          .write(b"VER\n")
          .read(concat!("ERR ", $error, "\n").as_bytes())
          .build();

        let mut client = nut_webgui_upsmc::client::NutClient::from(stream);

        match client.get_ver().await {
          Ok(_) => assert!(false, "Expected prot error received response"),
          Err(err) => match err.kind() {
            nut_webgui_upsmc::error::ErrorKind::IOError { .. } => {
              assert!(false, "Expected prot error received IO error")
            }
            nut_webgui_upsmc::error::ErrorKind::ParseError { .. } => {
              assert!(false, "Expected prot error received parse error")
            }
            nut_webgui_upsmc::error::ErrorKind::ProtocolError {
              inner: $expected,
            } => assert!(true),
            nut_webgui_upsmc::error::ErrorKind::ConnectionPoolClosed => {
              assert!(false, "Expected prot error received connection pool error")
            }
            _ => assert!(
              false,
              "Error kind is correct but inner prot error is not the expected one"
            ),
          },
        }
      }
    )+
  };
}

gen_prot_err_tests!(
  (prot_err_access_denied, ProtocolError::AccessDenied, "ACCESS-DENIED");
  (prot_err_already_attached, ProtocolError::AlreadyAttached, "ALREADY-ATTACHED");
  (prot_err_already_set_pass, ProtocolError::AlreadySetPassword, "ALREADY-SET-PASSWORD");
  (prot_err_already_set_user, ProtocolError::AlreadySetUsername, "ALREADY-SET-USERNAME");
  (prot_err_cmd_not_supported, ProtocolError::CmdNotSupported, "CMD-NOT-SUPPORTED");
  (prot_err_data_stale, ProtocolError::DateStale, "DATA-STALE");
  (prot_err_driver_not_connected, ProtocolError::DriverNotConnected, "DRIVER-NOT-CONNECTED");
  (prot_err_feature_not_configured, ProtocolError::FeatureNotConfigured, "FEATURE-NOT-CONFIGURED");
  (prot_err_feature_not_supported, ProtocolError::FeatureNotSupported, "FEATURE-NOT-SUPPORTED");
  (prot_err_instcmd_failed, ProtocolError::InstcmdFailed, "INSTCMD-FAILED");
  (prot_err_invalid_argument, ProtocolError::InvalidArgument, "INVALID-ARGUMENT");
  (prot_err_invalid_pass, ProtocolError::InvalidPassword, "INVALID-PASSWORD");
  (prot_err_invalid_user, ProtocolError::InvalidUsername, "INVALID-USERNAME");
  (prot_err_invalid_value, ProtocolError::InvalidValue, "INVALID-VALUE");
  (prot_err_pass_required, ProtocolError::PasswordRequired, "PASSWORD-REQUIRED");
  (prot_err_readonly, ProtocolError::Readonly, "READONLY");
  (prot_err_set_failed, ProtocolError::SetFailed, "SET-FAILED");
  (prot_err_tls_already_enabled, ProtocolError::TlsAlreadyEnabled, "TLS-ALREADY-ENABLED");
  (prot_err_tls_not_enabled, ProtocolError::TlsNotEnabled, "TLS-NOT-ENABLED");
  (prot_err_too_long ,ProtocolError::TooLong, "TOO-LONG");
  (prot_err_unknown_command, ProtocolError::UnknownCommand, "UNKNOWN-COMMAND");
  (prot_err_unknown_ups, ProtocolError::UnknownUps, "UNKNOWN-UPS");
  (prot_err_user_required, ProtocolError::UsernameRequired, "USERNAME-REQUIRED");
  (prot_err_var_not_supported, ProtocolError::VarNotSupported, "VAR-NOT-SUPPORTED");
  (prot_err_custom, ProtocolError::Unknown(..), "POTENTIAL-FUTURE-ERROR-NOT-DEFINED-IN-PROTOCOL");
);
