//! Log entries of the backend and the UI. Pure: no I/O.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub const MAX_MESSAGE_CHARS: usize = 4000;
pub const MAX_TARGET_CHARS: usize = 200;

/// Severity, most severe first. Debug and trace output never reaches the database.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Error,
    Warn,
    Info,
}

impl Level {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "error" => Self::Error,
            "warn" => Self::Warn,
            _ => Self::Info,
        }
    }

    /// This level and every more severe one.
    pub fn and_more_severe(self) -> Vec<Self> {
        [Self::Error, Self::Warn, Self::Info]
            .into_iter()
            .filter(|level| *level <= self)
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Backend,
    Frontend,
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Backend => "backend",
            Self::Frontend => "frontend",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "frontend" => Self::Frontend,
            _ => Self::Backend,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub id: i64,
    /// Unix milliseconds.
    pub created_at: i64,
    pub level: Level,
    pub source: Source,
    /// Rust module or UI area the entry comes from.
    pub target: String,
    pub message: String,
}

/// An entry to store; texts are cut to what the table accepts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewLogEntry {
    pub created_at: i64,
    pub level: Level,
    pub source: Source,
    pub target: String,
    pub message: String,
}

impl NewLogEntry {
    pub fn new(created_at: i64, level: Level, source: Source, target: &str, message: &str) -> Self {
        Self {
            created_at,
            level,
            source,
            target: truncate(target.trim(), MAX_TARGET_CHARS),
            message: truncate(message.trim(), MAX_MESSAGE_CHARS),
        }
    }
}

/// Which entries a listing shows, newest first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogQuery {
    /// Only entries at least this severe.
    pub min_level: Level,
    pub source: Option<Source>,
    pub limit: u32,
}

fn truncate(text: &str, max_chars: usize) -> String {
    match text.char_indices().nth(max_chars) {
        Some((end, _)) => format!("{}…", &text[..end]),
        None => text.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_levels_by_severity() {
        assert_eq!(Level::Error.and_more_severe(), [Level::Error]);
        assert_eq!(Level::Warn.and_more_severe(), [Level::Error, Level::Warn]);
        assert_eq!(Level::Info.and_more_severe().len(), 3);
        assert_eq!(Level::from_db(Level::Warn.as_str()), Level::Warn);
        assert_eq!(Source::from_db(Source::Frontend.as_str()), Source::Frontend);
    }

    #[test]
    fn cuts_long_texts_on_char_boundaries() {
        let entry = NewLogEntry::new(1, Level::Info, Source::Frontend, " ui ", &"ä".repeat(5000));
        assert_eq!(entry.target, "ui");
        assert_eq!(entry.message.chars().count(), MAX_MESSAGE_CHARS + 1);
        assert!(entry.message.ends_with('…'));
    }
}
