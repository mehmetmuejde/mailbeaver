//! Runtime statistics since the process started: what SMTP accepted and rejected and
//! how many messages were deleted. Kept in memory; they restart with the app.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

use crate::clock::now_millis;
use crate::db::{Database, DbError};
use utoipa::ToSchema;

#[derive(Debug, Default)]
struct Counters {
    accepted: AtomicU64,
    accepted_bytes: AtomicU64,
    rejected: AtomicU64,
    deleted: AtomicU64,
}

/// Cheap to clone; every clone counts into the same numbers.
#[derive(Debug, Clone)]
pub struct Stats {
    started_at: i64,
    counters: Arc<Counters>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatsSnapshot {
    /// Unix milliseconds.
    pub started_at: i64,
    pub smtp_accepted: u64,
    pub smtp_accepted_bytes: u64,
    pub smtp_rejected: u64,
    pub messages_deleted: u64,
    pub database_bytes: u64,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            started_at: now_millis(),
            counters: Arc::new(Counters::default()),
        }
    }

    pub fn record_accepted(&self, bytes: usize) {
        self.counters.accepted.fetch_add(1, Ordering::Relaxed);
        self.counters
            .accepted_bytes
            .fetch_add(u64::try_from(bytes).unwrap_or(u64::MAX), Ordering::Relaxed);
    }

    pub fn record_rejected(&self) {
        self.counters.rejected.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_deleted(&self, count: usize) {
        self.counters
            .deleted
            .fetch_add(u64::try_from(count).unwrap_or(u64::MAX), Ordering::Relaxed);
    }

    /// The counters plus the current size of the database file.
    pub async fn snapshot(&self, db: &Database) -> Result<StatsSnapshot, DbError> {
        let database_bytes = db
            .run(|conn| {
                let pages: i64 = conn.pragma_query_value(None, "page_count", |row| row.get(0))?;
                let page_size: i64 =
                    conn.pragma_query_value(None, "page_size", |row| row.get(0))?;
                Ok::<_, DbError>(u64::try_from(pages.saturating_mul(page_size)).unwrap_or(0))
            })
            .await?;
        Ok(StatsSnapshot {
            started_at: self.started_at,
            smtp_accepted: self.counters.accepted.load(Ordering::Relaxed),
            smtp_accepted_bytes: self.counters.accepted_bytes.load(Ordering::Relaxed),
            smtp_rejected: self.counters.rejected.load(Ordering::Relaxed),
            messages_deleted: self.counters.deleted.load(Ordering::Relaxed),
            database_bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn counts_and_measures_the_database() {
        let stats = Stats::new();
        stats.record_accepted(100);
        stats.record_accepted(50);
        stats.record_rejected();
        stats.record_deleted(3);
        let snapshot = stats
            .snapshot(&Database::open_in_memory().unwrap())
            .await
            .unwrap();
        assert_eq!(snapshot.smtp_accepted, 2);
        assert_eq!(snapshot.smtp_accepted_bytes, 150);
        assert_eq!(snapshot.smtp_rejected, 1);
        assert_eq!(snapshot.messages_deleted, 3);
        assert!(snapshot.database_bytes > 0);
    }
}
