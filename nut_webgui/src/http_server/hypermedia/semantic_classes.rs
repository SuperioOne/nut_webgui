#![allow(dead_code)]

pub const FILL_ERROR: &'static str = "fill-error";
pub const FILL_SUCCESS: &'static str = "fill-success";
pub const FILL_WARNING: &'static str = "fill-warning";
pub const FILL_INFO: &'static str = "fill-info";
pub const TEXT_ERROR: &'static str = "text-error";
pub const TEXT_SUCCESS: &'static str = "text-success";
pub const TEXT_WARNING: &'static str = "text-warning";
pub const TEXT_INFO: &'static str = "text-info";
pub const PROGRESS_ERROR: &'static str = "progress-error";
pub const PROGRESS_SUCCESS: &'static str = "progress-success";
pub const PROGRESS_WARNING: &'static str = "progress-warning";
pub const PROGRESS_INFO: &'static str = "progress-info";

static SEMANTIC_FILL_CLASSES: [&'static str; 3] = [FILL_SUCCESS, FILL_WARNING, FILL_ERROR];

static SEMANTIC_TEXT_CLASSES: [&'static str; 3] = [TEXT_SUCCESS, TEXT_WARNING, TEXT_ERROR];

static SEMANTIC_PROGRESS_CLASSES: [&'static str; 3] =
  [PROGRESS_SUCCESS, PROGRESS_WARNING, PROGRESS_ERROR];

#[inline]
fn get_class_idx(value: f64, from: f64, to: f64) -> usize {
  let inverted = from > to;
  ((value >= from) ^ inverted) as usize + ((value >= to) ^ inverted) as usize
}

pub fn get_fill_class(value: f64, from: f64, to: f64) -> &'static str {
  let idx = get_class_idx(value, from, to);
  unsafe { SEMANTIC_FILL_CLASSES.get_unchecked(idx) }
}

pub fn get_text_class(value: f64, from: f64, to: f64) -> &'static str {
  let idx = get_class_idx(value, from, to);
  unsafe { SEMANTIC_TEXT_CLASSES.get_unchecked(idx) }
}

pub fn get_progress_class(value: f64, from: f64, to: f64) -> &'static str {
  let idx = get_class_idx(value, from, to);
  unsafe { SEMANTIC_PROGRESS_CLASSES.get_unchecked(idx) }
}
