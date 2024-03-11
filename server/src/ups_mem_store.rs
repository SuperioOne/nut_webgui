use crate::upsd_client::ups_variables::UpsVariable;
use serde::Serialize;
use std::collections::hash_map::Iter;
use std::collections::HashMap;

#[derive(Debug)]
pub struct UpsEntry {
  pub name: Box<str>,
  pub desc: Box<str>,
  pub variables: Vec<UpsVariable>,
  pub commands: Vec<Box<str>>,
}

#[derive(Debug)]
pub(crate) struct UpsStore {
  ups_list: HashMap<Box<str>, UpsEntry>,
}

pub struct UpsStoreIterator<'a> {
  iterator: Iter<'a, Box<str>, UpsEntry>,
}

impl<'a> Iterator for UpsStoreIterator<'a> {
  type Item = (&'a Box<str>, &'a UpsEntry);

  fn next(&mut self) -> Option<Self::Item> {
    self.iterator.next()
  }
}

impl<'a> IntoIterator for &'a UpsStore {
  type Item = (&'a Box<str>, &'a UpsEntry);
  type IntoIter = UpsStoreIterator<'a>;

  fn into_iter(self) -> Self::IntoIter {
    UpsStoreIterator {
      iterator: self.ups_list.iter(),
    }
  }
}

impl UpsStore {
  pub fn new() -> UpsStore {
    UpsStore {
      ups_list: HashMap::new(),
    }
  }

  pub fn put(&mut self, entry: UpsEntry) {
    let key = entry.name.clone();
    self.ups_list.insert(key, entry);
  }

  pub fn get(&self, ups_name: &str) -> Option<&UpsEntry> {
    self.ups_list.get(ups_name)
  }

  pub fn get_mut(&mut self, ups_name: &str) -> Option<&mut UpsEntry> {
    self.ups_list.get_mut(ups_name)
  }
}
