//! Reading, writing and clearing the log. Entries of the UI are rate limited, so a
//! misbehaving page cannot flood the database.

use std::sync::{Arc, Mutex, PoisonError};

use crate::db::{Database, DbError};

use super::domain::{LogEntry, LogQuery, NewLogEntry};
use super::sqlite;

/// Entries kept in the table; older ones are dropped on the next write.
pub const KEEP_ENTRIES: u32 = 10_000;
/// Entries the UI may send per window.
const UI_ENTRIES_PER_WINDOW: u32 = 60;
const UI_WINDOW_MILLIS: i64 = 60_000;

#[derive(Debug, thiserror::Error)]
pub enum LogError {
    #[error("too many log entries, try again later")]
    RateLimited,
    #[error(transparent)]
    Db(#[from] DbError),
}

#[derive(Clone)]
pub struct LogService {
    db: Database,
    ui_window: Arc<Mutex<RateWindow>>,
}

#[derive(Debug, Default)]
struct RateWindow {
    started_at: i64,
    count: u32,
}

impl RateWindow {
    /// Counts one entry at `now`; `false` once the window is used up.
    fn admit(&mut self, now: i64) -> bool {
        if now - self.started_at >= UI_WINDOW_MILLIS {
            self.started_at = now;
            self.count = 0;
        }
        self.count += 1;
        self.count <= UI_ENTRIES_PER_WINDOW
    }
}

impl LogService {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            ui_window: Arc::default(),
        }
    }

    pub async fn list(&self, query: LogQuery) -> Result<Vec<LogEntry>, DbError> {
        self.db.run(move |conn| sqlite::list(conn, &query)).await
    }

    pub async fn clear(&self) -> Result<usize, DbError> {
        self.db.run(|conn| sqlite::clear(conn)).await
    }

    /// Stores entries of the backend, see [`super::persist`].
    pub async fn write(&self, entries: Vec<NewLogEntry>) -> Result<(), DbError> {
        self.db
            .run(move |conn| sqlite::insert(conn, &entries, KEEP_ENTRIES))
            .await
    }

    /// Stores an entry the UI sent.
    pub async fn record_from_ui(&self, entry: NewLogEntry) -> Result<(), LogError> {
        let admitted = self
            .ui_window
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .admit(entry.created_at);
        if !admitted {
            return Err(LogError::RateLimited);
        }
        Ok(self.write(vec![entry]).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_entries_per_window() {
        let mut window = RateWindow::default();
        let admitted = (0..70).filter(|_| window.admit(1_000)).count();
        assert_eq!(admitted, UI_ENTRIES_PER_WINDOW as usize);
        assert!(window.admit(1_000 + UI_WINDOW_MILLIS));
    }
}
