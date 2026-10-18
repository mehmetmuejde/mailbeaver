//! User settings: trash retention, tray grouping, display state, notifications, and the
//! ports and limits that apply at the next start.

mod domain;
mod http;
mod service;
mod sqlite;

pub use domain::{Grouping, InvalidServerSettings, Notifications, ServerSettings};
#[cfg(test)]
pub use domain::{Settings, Theme, TrashRetentionDays};
pub(crate) use http::router;
pub use service::{SettingsService, read_server_settings_at, save_server_settings_at};
