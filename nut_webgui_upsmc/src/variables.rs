use crate::{value::Value, var_name::VarName};
use std::collections::HashMap;

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

impl IntoIterator for UpsVariables {
  type Item = (VarName, Value);

  type IntoIter = IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    Self::IntoIter {
      inner_iter: self.inner.into_iter(),
    }
  }
}

pub struct IntoIter {
  inner_iter: std::collections::hash_map::IntoIter<VarName, Value>,
}

pub struct IterMut<'a> {
  inner_iter: std::collections::hash_map::IterMut<'a, VarName, Value>,
}

impl Iterator for IntoIter {
  type Item = (VarName, Value);

  fn next(&mut self) -> Option<Self::Item> {
    self.inner_iter.next()
  }
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
mod serde {
  use super::UpsVariables;
  use serde::de::Visitor;
  use std::collections::HashMap;

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

  struct UpsVariablesVisitor;

  impl<'de> serde::Deserialize<'de> for UpsVariables {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: serde::Deserializer<'de>,
    {
      deserializer.deserialize_map(UpsVariablesVisitor)
    }
  }

  impl<'de> Visitor<'de> for UpsVariablesVisitor {
    type Value = UpsVariables;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("expecting a ups variable map object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
      A: serde::de::MapAccess<'de>,
    {
      let mut vars = UpsVariables {
        inner: HashMap::with_capacity(map.size_hint().unwrap_or(0)),
      };

      while let Some((k, v)) = map.next_entry()? {
        vars.insert(k, v);
      }

      Ok(vars)
    }
  }
}
