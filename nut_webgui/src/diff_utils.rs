use nut_webgui_upsmc::{UpsName, responses::UpsDevice};
use std::{collections::HashMap, net::IpAddr};

pub trait Diff<T> {
  type Result;

  fn into_diff(self, target: T) -> Self::Result;
}

pub struct DeviceDiff {
  pub new: Vec<UpsDevice>,
  pub deleted: Vec<UpsName>,
  pub updated: Vec<UpsDevice>,
}

impl<I> Diff<I> for HashMap<UpsName, UpsDevice>
where
  I: IntoIterator<Item = UpsDevice>,
{
  type Result = DeviceDiff;

  fn into_diff(mut self, target: I) -> Self::Result {
    let mut result = DeviceDiff {
      new: Vec::new(),
      updated: Vec::new(),
      deleted: Vec::new(),
    };

    for device in target.into_iter() {
      match self.remove_entry(&device.ups_name) {
        Some((_, local_device)) => {
          if local_device.desc != device.desc {
            result.updated.push(device);
          }
        }
        None => {
          result.new.push(device);
        }
      }
    }

    if !self.is_empty() {
      result.deleted = self.into_values().map(|v| v.ups_name).collect();
    }

    result
  }
}

pub struct ClientDiff {
  pub connected: Vec<IpAddr>,
  pub disconnected: Vec<IpAddr>,
}

impl<'a, 'b> Diff<&'b [IpAddr]> for &'a [IpAddr]
where
  'a: 'b,
{
  type Result = ClientDiff;

  /// Generates diff from two slices of IPs. Current implementation expects extremly short
  /// slices.
  fn into_diff(self, target: &'_ [IpAddr]) -> Self::Result {
    let mut diff = ClientDiff {
      connected: Vec::new(),
      disconnected: Vec::new(),
    };

    for old in self.iter() {
      if target.iter().find(|v| *v == old).is_none() {
        diff.disconnected.push(*old);
      }
    }

    for new in target.iter() {
      if self.iter().find(|v| *v == new).is_none() {
        diff.connected.push(*new);
      }
    }

    diff
  }
}
