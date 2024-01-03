use std::fmt::{Display, Formatter};
use std::time::Duration;
use askama::Template;
use uuid::Uuid;

#[derive(Debug)]
pub enum Notification {
  Success,
  Warning,
  Error,
  Info,
}

impl Display for Notification {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Notification::Error => f.write_str("error"),
      Notification::Info => f.write_str("info"),
      Notification::Success => f.write_str("success"),
      Notification::Warning => f.write_str("warning"),
    }
  }
}

#[derive(Template)]
#[template(path = "notification.html", ext = "html")]
pub struct NotificationTemplate {
  message: String,
  ttl: u128,
  notification_type: Notification,
  container_id: String,
}

const DEFAULT_DURATION: Duration = Duration::from_millis(3000);

impl NotificationTemplate {
  /// Creates notification template with hx-swap-oob
  /// # Arguments
  ///
  /// * `message`: Notification body
  /// * `notification_type`: Notification level
  /// * `ttl`: Optional Time to live as milliseconds, Default is 3000ms.
  ///
  /// returns: NotificationTemplate
  pub fn new(message: String, notification_type: Notification, ttl: Option<Duration>) -> NotificationTemplate {
    let id = format!("notification_{}", Uuid::new_v4());
    let duration = ttl.unwrap_or(DEFAULT_DURATION);

    NotificationTemplate {
      message,
      notification_type,
      ttl: duration.as_millis(),
      container_id: id,
    }
  }
}

