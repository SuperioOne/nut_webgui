use super::semantic_type::SemanticType;

mod unit_si;
mod unit_wrapper;

pub use unit_si::*;
pub use unit_wrapper::*;

pub trait UnitDisplay: core::fmt::Display {
  type RawValue;

  fn get_semantic_type(&self) -> SemanticType;
  fn set_semantic_type(&mut self, semantic_type: SemanticType);
  fn get(&self) -> Self::RawValue;
}
