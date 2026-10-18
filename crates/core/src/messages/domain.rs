//! Captured messages: identities, folders, list queries and search. Pure: no I/O.

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::projects::ProjectId;

use super::mime::ParsedBody;
use super::search::SearchQuery;
use utoipa::ToSchema;

const MAX_AUTH_USER_CHARS: usize = 64;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema,
)]
#[serde(transparent)]
pub struct MessageId(pub i64);

/// Where a message currently lives (ADR-0008).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Folder {
    Inbox,
    Archive,
    Trash,
}

impl Folder {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inbox => "inbox",
            Self::Archive => "archive",
            Self::Trash => "trash",
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("unknown folder {0:?}")]
pub struct UnknownFolder(String);

impl FromStr for Folder {
    type Err = UnknownFolder;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "inbox" => Ok(Self::Inbox),
            "archive" => Ok(Self::Archive),
            "trash" => Ok(Self::Trash),
            other => Err(UnknownFolder(other.to_owned())),
        }
    }
}

/// The username an SMTP client authenticated with: trimmed, 1–64 characters, no control
/// characters. Kept as sent; project rules compare it without case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthUser(String);

impl AuthUser {
    pub fn parse(value: &str) -> Option<Self> {
        let value = value.trim();
        let length = value.chars().count();
        let valid =
            (1..=MAX_AUTH_USER_CHARS).contains(&length) && !value.chars().any(char::is_control);
        valid.then(|| Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// SMTP envelope of a received message.
#[derive(Debug, Clone, Default)]
pub struct Envelope {
    pub mail_from: String,
    pub rcpt_to: Vec<String>,
    pub auth_user: Option<AuthUser>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRef {
    pub id: ProjectId,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessageSummary {
    pub id: MessageId,
    /// Unix timestamp in milliseconds.
    pub received_at: i64,
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub size: i64,
    pub has_attachments: bool,
    pub is_read: bool,
    pub folder: Folder,
    pub project: Option<ProjectRef>,
    /// The start of the plain-text body, whitespace collapsed.
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessagePage {
    /// Messages matching the query, across all pages.
    pub total: i64,
    pub unread: i64,
    pub messages: Vec<MessageSummary>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessageDetail {
    #[serde(flatten)]
    pub summary: MessageSummary,
    pub envelope_from: String,
    pub envelope_to: Vec<String>,
    pub auth_user: Option<String>,
    #[serde(flatten)]
    pub body: ParsedBody,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Counts {
    pub total: i64,
    pub unread: i64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FolderCounts {
    pub inbox: Counts,
    pub archive: Counts,
    pub trash: Counts,
}

/// Which projects a listing covers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ProjectFilter {
    #[default]
    All,
    /// Messages no rule assigns to a project.
    Unassigned,
    Project(ProjectId),
}

#[derive(Debug, Clone, Default)]
pub struct ListQuery {
    /// `None` lists all folders.
    pub folder: Option<Folder>,
    /// Leaves Trash out when listing all folders, e.g. for the Mailpit view.
    pub exclude_trash: bool,
    pub project: ProjectFilter,
    pub search: Option<SearchQuery>,
    /// Only messages received at or after this Unix millisecond.
    pub since: Option<i64>,
    pub limit: u32,
    pub offset: u32,
}

/// Result of a permanent delete, which only Trash allows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteOutcome {
    Deleted,
    NotFound,
    NotInTrash,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_auth_users() {
        assert_eq!(AuthUser::parse("  Shop ").unwrap().as_str(), "Shop");
        assert!(AuthUser::parse("   ").is_none());
        assert!(AuthUser::parse("a\u{0}b").is_none());
        assert!(AuthUser::parse(&"x".repeat(65)).is_none());
        assert!(AuthUser::parse(&"ä".repeat(64)).is_some());
    }
}
