use core::{
  cmp::Ordering,
  hash::Hash,
  ptr::null,
  str::{FromStr, from_utf8_unchecked},
};
use std::ffi::CString;

#[derive(Clone, Copy, Debug)]
pub struct InvalidDbNameError;

#[derive(Clone, Debug)]
pub struct DbName {
  name: Option<CString>,
}

impl DbName {
  pub const UNNAMED: DbName = DbName { name: None };

  /// Return database name as C-style poiter.
  ///
  /// **NOTE:** This function returns NULL when name is empty. LMDB expects
  /// NULL as unnamed database name.
  pub(crate) unsafe fn as_ptr(&self) -> *const i8 {
    if let Some(v) = self.name.as_ref()
      && !v.is_empty()
    {
      v.as_ptr()
    } else {
      null()
    }
  }

  #[inline]
  pub fn new<T>(name: T) -> Result<Self, InvalidDbNameError>
  where
    T: AsRef<str>,
  {
    name.as_ref().try_into()
  }

  pub fn new_empty() -> Self {
    Self { name: None }
  }

  /// Creates new DbName without any check. Caller is responsible to create
  /// proper name str without any null bytes.
  pub unsafe fn from_static(name: &'static str) -> Self {
    let buf: Vec<u8> = Vec::from(name.as_bytes());
    let name = unsafe { CString::from_vec_unchecked(buf) };

    Self { name: Some(name) }
  }

  pub fn as_str(&self) -> &str {
    match self.name.as_ref() {
      Some(v) => {
        // This call is currently safe since DbName cannot be constructed from
        // non-Utf8 bytes (Unless some masochistics create a str from random ass
        // bytes)
        unsafe { from_utf8_unchecked(v.as_bytes()) }
      }
      None => "",
    }
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.name.as_ref().is_none_or(|v| v.is_empty())
  }
}

impl TryFrom<&str> for DbName {
  type Error = InvalidDbNameError;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    let name = CString::from_str(value).map_err(|_| InvalidDbNameError)?;
    Ok(Self { name: Some(name) })
  }
}

impl TryFrom<String> for DbName {
  type Error = InvalidDbNameError;

  #[inline]
  fn try_from(value: String) -> Result<Self, Self::Error> {
    value.as_str().try_into()
  }
}

impl FromStr for DbName {
  type Err = InvalidDbNameError;

  #[inline]
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    s.try_into()
  }
}

impl PartialEq for DbName {
  fn eq(&self, other: &Self) -> bool {
    match (self.name.as_ref(), other.name.as_ref()) {
      (None, None) => true,
      (None, Some(l)) => l.is_empty(),
      (Some(r), None) => r.is_empty(),
      (Some(l), Some(r)) => l == r,
    }
  }
}

impl Eq for DbName {}

impl Ord for DbName {
  fn cmp(&self, other: &Self) -> Ordering {
    match (self.name.as_ref(), other.name.as_ref()) {
      (None, None) => Ordering::Equal,
      (None, Some(l)) => {
        if l.is_empty() {
          Ordering::Equal
        } else {
          Ordering::Less
        }
      }
      (Some(r), None) => {
        if r.is_empty() {
          Ordering::Equal
        } else {
          Ordering::Greater
        }
      }
      (Some(r), Some(l)) => r.cmp(l),
    }
  }
}

impl PartialOrd for DbName {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Hash for DbName {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    match self.name.as_ref() {
      Some(v) => {
        if v.is_empty() {
          state.write_u8(0);
        } else {
          v.hash(state);
        }
      }
      None => state.write_u8(0),
    }
  }
}

impl Default for DbName {
  #[inline]
  fn default() -> Self {
    Self::UNNAMED
  }
}

impl core::fmt::Display for InvalidDbNameError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str("database name with null-terminators are not allowed")
  }
}

impl core::error::Error for InvalidDbNameError {}

#[cfg(test)]
mod test {
  use super::DbName;
  use std::collections::{BTreeMap, BTreeSet, hash_map};
  use std::hash::{Hash, Hasher};

  #[test]
  fn new_rejects_null_byte() {
    assert!(DbName::new("a\0b").is_err());
    assert!(DbName::new("\0").is_err());
    assert!(DbName::new("a\0").is_err());
    assert!(DbName::new("a\0b").is_err());
  }

  #[test]
  fn new_accepts_empty_string() {
    let name = DbName::new("").unwrap();
    assert!(name.is_empty());
    assert_eq!(name, DbName::UNNAMED);
  }

  #[test]
  fn new_accepts_unicode() {
    let name = DbName::new("databases/かちかち山").unwrap();
    assert_eq!(name.as_str(), "databases/かちかち山");
  }

  #[test]
  fn new_accepts_from_string() {
    let name: DbName = String::from("mydb").try_into().unwrap();
    assert_eq!(name.as_str(), "mydb");
  }

  #[test]
  fn eq_unnamed_and_empty_string() {
    let unnamed = DbName::UNNAMED;
    let empty = DbName::new("").unwrap();
    assert_eq!(unnamed, empty);
    assert_eq!(empty, unnamed);
  }

  #[test]
  fn eq_unnamed_and_new_empty() {
    assert_eq!(DbName::UNNAMED, DbName::new_empty());
    assert_eq!(DbName::new_empty(), DbName::UNNAMED);
  }

  #[test]
  fn eq_identical_names() {
    assert_eq!(DbName::new("foo").unwrap(), DbName::new("foo").unwrap());
  }

  #[test]
  fn ne_different_names() {
    assert_ne!(DbName::new("foo").unwrap(), DbName::new("bar").unwrap());
    assert_ne!(DbName::new("foo").unwrap(), DbName::UNNAMED);
    assert_ne!(DbName::UNNAMED, DbName::new("foo").unwrap());
  }

  #[test]
  fn eq_is_reflexive() {
    let a = DbName::new("hello").unwrap();
    let b = DbName::UNNAMED;
    assert_eq!(a, a);
    assert_eq!(b, b);
  }

  #[test]
  fn ord_unnamed_less_than_named() {
    assert!(DbName::UNNAMED < DbName::new("a").unwrap());
    assert!(DbName::new("a").unwrap() > DbName::UNNAMED);
  }

  #[test]
  fn ord_named_ordering() {
    let a = DbName::new("a").unwrap();
    let b = DbName::new("b").unwrap();
    let c = DbName::new("c").unwrap();
    assert!(a < b);
    assert!(b < c);
    assert!(a < c);
    assert!(b > a);
    assert!(c > b);
  }

  #[test]
  fn ord_equal_implies_cmp_equal_unnamed_vs_empty() {
    let unnamed = DbName::UNNAMED;
    let empty = DbName::new("").unwrap();
    assert_eq!(unnamed, empty);
    assert_eq!(unnamed.cmp(&empty), std::cmp::Ordering::Equal);
    assert_eq!(empty.cmp(&unnamed), std::cmp::Ordering::Equal);
  }

  fn hash_of<T: Hash>(v: &T) -> u64 {
    let mut h = hash_map::DefaultHasher::new();
    v.hash(&mut h);
    h.finish()
  }

  #[test]
  fn hash_equal_values_same_hash() {
    let a = DbName::UNNAMED;
    let b = DbName::new_empty();
    let c = DbName::new("").unwrap();
    assert_eq!(hash_of(&a), hash_of(&b),);
    assert_eq!(hash_of(&a), hash_of(&c),);
    assert_eq!(hash_of(&b), hash_of(&c));
  }

  #[test]
  fn hash_different_names_different_hash() {
    let a = DbName::new("foo").unwrap();
    let b = DbName::new("bar").unwrap();
    assert_ne!(hash_of(&a), hash_of(&b));
  }

  #[test]
  fn hash_collide() {
    let mut map: BTreeMap<DbName, i32> = BTreeMap::new();
    let empty = DbName::new("").unwrap();
    map.insert(DbName::UNNAMED, 1);
    map.insert(empty.clone(), 2);
    assert_eq!(map.len(), 1,);
    assert_eq!(map[&DbName::UNNAMED], 2);
    assert_eq!(map[&empty], 2);
  }

  #[test]
  fn hash_ordering() {
    let mut map: BTreeSet<DbName> = BTreeSet::new();
    map.insert(DbName::new("never gonna").unwrap());
    map.insert(DbName::new("give you up").unwrap());
    map.insert(DbName::UNNAMED);
    map.insert(DbName::new("let you down").unwrap());

    let keys: Vec<&str> = map.iter().map(|k| k.as_str()).collect();
    assert_eq!(keys, vec!["", "give you up", "let you down", "never gonna"]);
  }
}
