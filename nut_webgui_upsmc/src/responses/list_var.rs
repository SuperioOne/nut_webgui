use crate::{UpsName, UpsVar};

pub struct ListVar {
  pub variables: Vec<UpsVar>,
  pub ups: UpsName,
}
