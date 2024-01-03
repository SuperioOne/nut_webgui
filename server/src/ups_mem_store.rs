use std::collections::{HashMap};
use std::collections::hash_map::Iter;
use serde::{Serialize};
use crate::upsd_client::protocol::UpsVariable;

#[derive(Debug, Serialize)]
pub struct UpsEntry {
  pub name: Box<str>,
  pub desc: Box<str>,
  pub variables: Vec<UpsVariable>,
  pub commands: Vec<Box<str>>,
}

#[derive(Debug)]
pub(crate) struct UpsStore
{
  name: Box<str>,
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
      iterator: self.ups_list.iter()
    }
  }
}

impl UpsStore {
  pub fn new(name: &str) -> UpsStore {
    UpsStore {
      ups_list: HashMap::new(),
      name: Box::from(name),
    }
  }

  pub fn create_or_update(&mut self, entry: UpsEntry)
  {
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