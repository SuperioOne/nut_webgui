use crate::{UpsName, UpsVar};

pub struct ListRW {
  pub rw_variables: Vec<UpsVar>,
  pub ups: UpsName,
}
