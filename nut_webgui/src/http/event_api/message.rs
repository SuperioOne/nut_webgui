use super::error::HandshakeError;
use crate::state::ConnectionStatus;
use axum::extract::ws::Message;
use nut_webgui_upsmc::{UpsName, ups_event::UpsEvents, ups_status::UpsStatus};
use serde::Serialize;
use std::net::IpAddr;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum NutEventMessage<'a> {
  DeviceConnected {
    name: &'a UpsName,
    namespace: &'a str,
    timestamp: i64,
  },
  DeviceRemoved {
    name: &'a UpsName,
    namespace: &'a str,
    timestamp: i64,
  },
  DeviceUpdate {
    name: &'a UpsName,
    namespace: &'a str,
    timestamp: i64,
  },
  DeviceStatus {
    name: &'a UpsName,
    namespace: &'a str,
    status_new: UpsStatus,
    status_old: UpsStatus,
    events: UpsEvents,
    timestamp: i64,
  },
  DaemonStatus {
    namespace: &'a str,
    status: ConnectionStatus,
    timestamp: i64,
  },
  ClientConnect {
    client_ip: IpAddr,
    name: &'a UpsName,
    namespace: &'a str,
    timestamp: i64,
  },
  ClientDisconnect {
    client_ip: IpAddr,
    name: &'a UpsName,
    namespace: &'a str,
    timestamp: i64,
  },
  HandshakeError {
    message: &'a HandshakeError,
  },
  SessionEnded,
  WaitingForAuth,
  AuthOk,
}

impl TryInto<Message> for NutEventMessage<'_> {
  type Error = serde_json::error::Error;

  fn try_into(self) -> Result<Message, Self::Error> {
    let data = serde_json::to_string(&self)?;
    Ok(Message::text(data))
  }
}
