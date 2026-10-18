//! User settings. Pure: no I/O.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Days a message stays in Trash before it is deleted automatically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "u16", into = "u16")]
#[schema(value_type = u16, minimum = 1, maximum = 365)]
pub struct TrashRetentionDays(u16);

impl TrashRetentionDays {
    pub const MAX: u16 = 365;

    pub fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, thiserror::Error)]
#[error(
    "trash retention must be between 1 and {} days",
    TrashRetentionDays::MAX
)]
pub struct InvalidRetention;

impl TryFrom<u16> for TrashRetentionDays {
    type Error = InvalidRetention;

    fn try_from(days: u16) -> Result<Self, Self::Error> {
        (1..=Self::MAX)
            .contains(&days)
            .then_some(Self(days))
            .ok_or(InvalidRetention)
    }
}

impl From<TrashRetentionDays> for u16 {
    fn from(days: TrashRetentionDays) -> Self {
        days.0
    }
}

/// How the tray menu groups the newest inbox messages.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Grouping {
    #[default]
    All,
    Sender,
    Project,
}

impl Grouping {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Sender => "sender",
            Self::Project => "project",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "sender" => Self::Sender,
            "project" => Self::Project,
            _ => Self::All,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings {
    /// `None` keeps messages in Trash until it is emptied by hand.
    pub trash_retention_days: Option<TrashRetentionDays>,
    pub tray_grouping: Grouping,
    pub theme: Theme,
    pub sidebar_collapsed: bool,
    /// Which new mail raises a system notification (desktop app).
    pub notifications: Notifications,
    pub notification_sound: bool,
    /// Unread inbox count on the app icon (desktop app).
    pub unread_badge: bool,
    /// Asks GitHub once a day for a newer release (ADR-0032).
    pub update_check: bool,
}

/// The values of a fresh database.
impl Default for Settings {
    fn default() -> Self {
        Self {
            trash_retention_days: None,
            tray_grouping: Grouping::default(),
            theme: Theme::default(),
            sidebar_collapsed: false,
            notifications: Notifications::default(),
            notification_sound: false,
            unread_badge: true,
            update_check: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Notifications {
    Off,
    #[default]
    All,
    /// Only mail of projects that have notifications switched on.
    Projects,
}

impl Notifications {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::All => "all",
            Self::Projects => "projects",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "off" => Self::Off,
            "projects" => Self::Projects,
            _ => Self::All,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// Follows the operating system.
    #[default]
    System,
    Light,
    Dark,
}

impl Theme {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "light" => Self::Light,
            "dark" => Self::Dark,
            _ => Self::System,
        }
    }
}

/// Ports and limits saved in the app (ADR-0031); `None` keeps the default. They apply
/// at the next start, and an environment variable for the same value wins.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServerSettings {
    #[schema(minimum = 1, maximum = 65535)]
    pub smtp_port: Option<u16>,
    #[schema(minimum = 1, maximum = 65535)]
    pub http_port: Option<u16>,
    #[schema(minimum = 1, maximum = 1_000_000)]
    pub max_messages: Option<u32>,
    #[schema(minimum = 1, maximum = 1024)]
    pub max_message_mb: Option<u32>,
}

pub const MAX_STORED_MESSAGES: u32 = 1_000_000;
pub const MAX_MESSAGE_MB: u32 = 1024;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidServerSettings {
    #[error("ports must be between 1 and 65535")]
    Port,
    #[error("SMTP and HTTP need different ports")]
    SamePort,
    #[error("keep between 1 and {MAX_STORED_MESSAGES} messages")]
    MaxMessages,
    #[error("the message size limit must be between 1 and {MAX_MESSAGE_MB} MB")]
    MaxMessageMb,
}

impl ServerSettings {
    pub fn validate(&self) -> Result<(), InvalidServerSettings> {
        if self.smtp_port == Some(0) || self.http_port == Some(0) {
            return Err(InvalidServerSettings::Port);
        }
        if self.smtp_port.is_some() && self.smtp_port == self.http_port {
            return Err(InvalidServerSettings::SamePort);
        }
        if self
            .max_messages
            .is_some_and(|max| !(1..=MAX_STORED_MESSAGES).contains(&max))
        {
            return Err(InvalidServerSettings::MaxMessages);
        }
        if self
            .max_message_mb
            .is_some_and(|mb| !(1..=MAX_MESSAGE_MB).contains(&mb))
        {
            return Err(InvalidServerSettings::MaxMessageMb);
        }
        Ok(())
    }
}

/// Time range a folder list is limited to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum Period {
    #[serde(rename = "today")]
    Today,
    #[serde(rename = "7d")]
    SevenDays,
    #[serde(rename = "30d")]
    ThirtyDays,
}

impl Period {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Today => "today",
            Self::SevenDays => "7d",
            Self::ThirtyDays => "30d",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "today" => Some(Self::Today),
            "7d" => Some(Self::SevenDays),
            "30d" => Some(Self::ThirtyDays),
            _ => None,
        }
    }
}

/// Messages per page of a folder list: 25, 50 or 100.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "u16", into = "u16")]
#[schema(value_type = u16)]
pub struct PageSize(u16);

impl PageSize {
    const ALLOWED: [u16; 3] = [25, 50, 100];

    pub fn get(self) -> u16 {
        self.0
    }
}

impl Default for PageSize {
    fn default() -> Self {
        Self(50)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("page size must be 25, 50 or 100")]
pub struct InvalidPageSize;

impl TryFrom<u16> for PageSize {
    type Error = InvalidPageSize;

    fn try_from(size: u16) -> Result<Self, Self::Error> {
        Self::ALLOWED
            .contains(&size)
            .then_some(Self(size))
            .ok_or(InvalidPageSize)
    }
}

impl From<PageSize> for u16 {
    fn from(size: PageSize) -> Self {
        size.0
    }
}

/// A project filter: `none` (mail without a project) or a project id as text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "String", into = "String")]
#[schema(value_type = String, example = "none")]
pub struct ProjectChoice(String);

#[derive(Debug, thiserror::Error)]
#[error("project must be \"none\" or a project id")]
pub struct InvalidProjectChoice;

impl TryFrom<String> for ProjectChoice {
    type Error = InvalidProjectChoice;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let valid = value == "none" || value.parse::<i64>().is_ok_and(|id| id > 0);
        valid.then_some(Self(value)).ok_or(InvalidProjectChoice)
    }
}

impl From<ProjectChoice> for String {
    fn from(choice: ProjectChoice) -> Self {
        choice.0
    }
}

impl ProjectChoice {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// How one folder was last shown: grouping, filters and page size.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FolderView {
    pub grouping: Grouping,
    /// `None` shows every project.
    pub project: Option<ProjectChoice>,
    /// `None` shows any time.
    pub period: Option<Period>,
    pub page_size: PageSize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FolderViews {
    pub inbox: FolderView,
    pub archive: FolderView,
    pub trash: FolderView,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_server_settings() {
        let ports = |smtp, http| ServerSettings {
            smtp_port: smtp,
            http_port: http,
            ..ServerSettings::default()
        };
        assert_eq!(ServerSettings::default().validate(), Ok(()));
        assert_eq!(ports(Some(2525), Some(8080)).validate(), Ok(()));
        assert_eq!(
            ports(Some(2525), Some(2525)).validate(),
            Err(InvalidServerSettings::SamePort)
        );
        assert_eq!(
            ports(Some(0), None).validate(),
            Err(InvalidServerSettings::Port)
        );
        let limits = ServerSettings {
            max_messages: Some(0),
            ..ServerSettings::default()
        };
        assert_eq!(limits.validate(), Err(InvalidServerSettings::MaxMessages));
        let size = ServerSettings {
            max_message_mb: Some(2048),
            ..ServerSettings::default()
        };
        assert_eq!(size.validate(), Err(InvalidServerSettings::MaxMessageMb));
    }

    #[test]
    fn retention_stays_in_range() {
        assert!(TrashRetentionDays::try_from(0).is_err());
        assert!(TrashRetentionDays::try_from(366).is_err());
        assert_eq!(TrashRetentionDays::try_from(30).unwrap().get(), 30);
    }

    #[test]
    fn validates_folder_views() {
        let view: FolderView = serde_json::from_str(
            r#"{"grouping":"sender","project":"12","period":"7d","pageSize":100}"#,
        )
        .unwrap();
        assert_eq!(view.page_size.get(), 100);
        assert_eq!(view.period, Some(Period::SevenDays));
        for bad in [
            r#"{"grouping":"all","project":"shop","period":null,"pageSize":50}"#,
            r#"{"grouping":"all","project":"0","period":null,"pageSize":50}"#,
            r#"{"grouping":"all","project":null,"period":"1y","pageSize":50}"#,
            r#"{"grouping":"all","project":null,"period":null,"pageSize":40}"#,
        ] {
            assert!(serde_json::from_str::<FolderView>(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn accepts_complete_settings() {
        let settings: Settings = serde_json::from_str(
            r#"{"trashRetentionDays":null,"trayGrouping":"all","theme":"system","sidebarCollapsed":false,"notifications":"projects","notificationSound":true,"unreadBadge":false,"updateCheck":true}"#,
        )
        .unwrap();
        assert_eq!(settings.notifications, Notifications::Projects);
        assert!(
            serde_json::from_str::<Settings>(
                r#"{"trashRetentionDays":null,"trayGrouping":"all","theme":"system","sidebarCollapsed":false,"notifications":"some","notificationSound":true,"unreadBadge":false,"updateCheck":true}"#
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_unknown_fields_and_values() {
        assert!(
            serde_json::from_str::<Settings>(r#"{"trashRetentionDays":0,"trayGrouping":"all","theme":"system","sidebarCollapsed":false,"notifications":"all","notificationSound":false,"unreadBadge":true,"updateCheck":false}"#)
                .is_err()
        );
        assert!(
            serde_json::from_str::<Settings>(
                r#"{"trashRetentionDays":null,"trayGrouping":"size","theme":"system","sidebarCollapsed":false,"notifications":"all","notificationSound":false,"unreadBadge":true,"updateCheck":false}"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<Settings>(
                r#"{"trashRetentionDays":null,"trayGrouping":"all","theme":"system","sidebarCollapsed":false,"notifications":"all","notificationSound":false,"unreadBadge":true,"updateCheck":false,"x":1}"#
            )
            .is_err()
        );
    }
}
