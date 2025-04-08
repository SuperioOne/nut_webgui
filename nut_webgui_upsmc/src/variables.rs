use std::collections::HashMap;
use std::collections::hash_map::{Iter, IterMut};

use crate::value::Value;
use crate::var_name::VarName;

/// List of UPS variables.
///
/// ## Implementation notes:
/// It simply a wrapper struct for [`HashMap`].
pub struct UpsVariables {
  inner: HashMap<VarName, Value>,
}

impl UpsVariables {
  pub fn new() -> Self {
    Self {
      inner: HashMap::new(),
    }
  }

  pub fn insert(&mut self, name: VarName, value: Value) -> Option<Value> {
    self.inner.insert(name, value)
  }

  pub fn remove(&mut self, name: VarName) -> Option<(VarName, Value)> {
    self.inner.remove_entry(&name)
  }

  pub fn len(&self) -> usize {
    self.inner.len()
  }

  pub fn iter_mut(&mut self) -> UpsVariablesIterMut<'_> {
    UpsVariablesIterMut {
      inner_iter: self.inner.iter_mut(),
    }
  }

  pub fn iter(&mut self) -> UpsVariablesIter<'_> {
    UpsVariablesIter {
      inner_iter: self.inner.iter(),
    }
  }
}

pub struct UpsVariablesIterMut<'a> {
  inner_iter: IterMut<'a, VarName, Value>,
}

impl<'a> Iterator for UpsVariablesIterMut<'a> {
  type Item = (&'a VarName, &'a mut Value);

  fn next(&mut self) -> Option<Self::Item> {
    self.inner_iter.next()
  }
}

pub struct UpsVariablesIter<'a> {
  inner_iter: Iter<'a, VarName, Value>,
}

impl<'a> Iterator for UpsVariablesIter<'a> {
  type Item = (&'a VarName, &'a Value);

  fn next(&mut self) -> Option<Self::Item> {
    self.inner_iter.next()
  }
}

#[cfg(feature = "serde")]
impl serde::Serialize for UpsVariables {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    use serde::ser::SerializeMap;

    let mut map_serializer = serializer.serialize_map(Some(self.inner.len()))?;

    for (key, value) in self.inner.iter() {
      map_serializer.serialize_entry(key, value)?;
    }

    map_serializer.end()
  }
}
