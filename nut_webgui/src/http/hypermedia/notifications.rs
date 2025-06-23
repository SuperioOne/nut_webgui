use crate::http::hypermedia::semantic_classes::SemanticType;
use askama::Template;
use core::time::Duration;
use std::borrow::Cow;

#[derive(Template, Debug)]
#[template(path = "notification.html", ext = "html")]
pub struct NotificationTemplate<'a> {
  pub message: Cow<'a, str>,
  pub ttl: u128,
  pub semantic_type: SemanticType,
}

const DEFAULT_DURATION_MS: u128 = 3000;

impl<'a> NotificationTemplate<'a> {
  #[inline]
  pub fn set_level(mut self, level: SemanticType) -> Self {
    self.semantic_type = level;
    self
  }

  #[inline]
  pub fn set_ttl(mut self, ttl: Duration) -> Self {
    self.ttl = ttl.as_millis();
    self
  }
}

impl<'a> From<&'a str> for NotificationTemplate<'a> {
  #[inline]
  fn from(value: &'a str) -> Self {
    NotificationTemplate {
      message: Cow::Borrowed(value),
      ttl: DEFAULT_DURATION_MS,
      semantic_type: SemanticType::Info,
    }
  }
}

impl From<String> for NotificationTemplate<'_> {
  #[inline]
  fn from(value: String) -> Self {
    NotificationTemplate {
      message: Cow::Owned(value),
      ttl: DEFAULT_DURATION_MS,
      semantic_type: SemanticType::Info,
    }
  }
}
