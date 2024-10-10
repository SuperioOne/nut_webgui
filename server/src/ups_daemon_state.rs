use crate::upsd_client::ups_variables::UpsVariable;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::{hash_map::Iter, HashMap};
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize)]
pub enum DaemonStatus {
  Dead,
  Online,
  NotReady,
}

#[derive(Debug)]
pub struct UpsEntry {
  pub name: Box<str>,
  pub desc: Box<str>,
  pub variables: Vec<UpsVariable>,
  pub commands: Vec<Box<str>>,
}

#[derive(Debug)]
pub struct UpsDaemonState {
  pub ups_list: HashMap<Box<str>, UpsEntry>,
  pub last_full_sync: Option<DateTime<Utc>>,
  pub last_modified: Option<DateTime<Utc>>,
  pub status: DaemonStatus,
}

pub struct UpsDaemonIterator<'a> {
  iterator: Iter<'a, Box<str>, UpsEntry>,
}

impl<'a> Iterator for UpsDaemonIterator<'a> {
  type Item = (&'a Box<str>, &'a UpsEntry);

  fn next(&mut self) -> Option<Self::Item> {
    self.iterator.next()
  }
}

impl Display for DaemonStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DaemonStatus::Dead => f.write_str("Dead"),
      DaemonStatus::Online => f.write_str("Online"),
      DaemonStatus::NotReady => f.write_str("Not Ready"),
    }
  }
}

impl UpsDaemonState {
  pub fn new() -> UpsDaemonState {
    UpsDaemonState {
      ups_list: HashMap::new(),
      last_full_sync: None,
      last_modified: None,
      status: DaemonStatus::NotReady,
    }
  }

  pub fn put_ups(&mut self, entry: UpsEntry) {
    let key = entry.name.clone();
    self.ups_list.insert(key, entry);
  }

  pub fn get_ups(&self, ups_name: &str) -> Option<&UpsEntry> {
    self.ups_list.get(ups_name)
  }

  pub fn get_ups_mut(&mut self, ups_name: &str) -> Option<&mut UpsEntry> {
    self.ups_list.get_mut(ups_name)
  }

  pub fn iter(&self) -> UpsDaemonIterator<'_> {
    UpsDaemonIterator {
      iterator: self.ups_list.iter(),
    }
  }

  pub fn reset(&mut self) {
    self.ups_list.clear();
    self.last_modified = None;
    self.last_full_sync = None;
    self.status = DaemonStatus::NotReady;
  }

  pub fn reset_with_status(&mut self, status: DaemonStatus) {
    self.ups_list.clear();
    self.last_modified = None;
    self.last_full_sync = None;
    self.status = status;
  }
}
