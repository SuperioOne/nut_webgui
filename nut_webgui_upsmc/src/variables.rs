use std::collections::HashMap;

use crate::value::Value;
use crate::var_name::VarName;

/// List of UPS variables.
///
/// ## Implementation notes:
/// It simply a wrapper struct for [`HashMap`].
#[derive(Debug, Clone)]
pub struct UpsVariables {
  inner: HashMap<VarName, Value>,
}

impl UpsVariables {
  pub fn new() -> Self {
    Self {
      inner: HashMap::new(),
    }
  }

  pub fn get(&self, name: &VarName) -> Option<&Value> {
    self.inner.get(name)
  }

  pub fn get_mut(&mut self, name: &VarName) -> Option<&mut Value> {
    self.inner.get_mut(name)
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

  pub fn iter_mut(&mut self) -> IterMut<'_> {
    IterMut {
      inner_iter: self.inner.iter_mut(),
    }
  }

  pub fn iter(&self) -> Iter<'_> {
    Iter {
      inner_iter: self.inner.iter(),
    }
  }
}

pub struct IterMut<'a> {
  inner_iter: std::collections::hash_map::IterMut<'a, VarName, Value>,
}

impl<'a> Iterator for IterMut<'a> {
  type Item = (&'a VarName, &'a mut Value);

  fn next(&mut self) -> Option<Self::Item> {
    self.inner_iter.next()
  }
}

pub struct Iter<'a> {
  inner_iter: std::collections::hash_map::Iter<'a, VarName, Value>,
}

impl<'a> Iterator for Iter<'a> {
  type Item = (&'a VarName, &'a Value);

  fn next(&mut self) -> Option<Self::Item> {
    self.inner_iter.next()
  }
}

impl<const N: usize> From<[(VarName, Value); N]> for UpsVariables {
  fn from(value: [(VarName, Value); N]) -> Self {
    Self {
      inner: HashMap::from(value),
    }
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
