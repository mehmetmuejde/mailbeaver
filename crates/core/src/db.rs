//! The SQLite database every feature stores its data in: one connection behind a mutex,
//! the schema migrations and the error type all storage code returns.
//!
//! Store calls are synchronous; async callers go through [`Database::run`], which moves
//! the work to the blocking thread pool.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rusqlite::Connection;
use tokio::task::JoinError;

/// Applied in order; `PRAGMA user_version` records how many have run.
const MIGRATIONS: [&str; 3] = [
    include_str!("db/migrations/0001_initial.sql"),
    include_str!("db/migrations/0002_server_settings.sql"),
    include_str!("db/migrations/0003_update_check.sql"),
];

const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database error")]
    Sqlite(#[from] rusqlite::Error),
    #[error("database schema version {0} is newer than this build supports")]
    UnsupportedSchema(i32),
    #[error("database lock poisoned by a panicked thread")]
    Poisoned,
    #[error("background task failed")]
    Task(#[from] JoinError),
}

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, DbError> {
        Self::init(Connection::open(path)?)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, DbError> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(mut conn: Connection) -> Result<Self, DbError> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.busy_timeout(BUSY_TIMEOUT)?;
        migrate(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Runs `work` with exclusive access to the connection, on the calling thread.
    /// Features may return their own error type as long as it wraps [`DbError`].
    pub fn with<T, E>(&self, work: impl FnOnce(&mut Connection) -> Result<T, E>) -> Result<T, E>
    where
        E: From<DbError>,
    {
        let mut conn = self.conn.lock().map_err(|_| DbError::Poisoned)?;
        work(&mut conn)
    }

    /// Runs `work` on the blocking thread pool so async callers never block a runtime
    /// thread.
    pub async fn run<T, E, F>(&self, work: F) -> Result<T, E>
    where
        F: FnOnce(&mut Connection) -> Result<T, E> + Send + 'static,
        T: Send + 'static,
        E: From<DbError> + Send + 'static,
    {
        let db = self.clone();
        tokio::task::spawn_blocking(move || db.with(work))
            .await
            .map_err(DbError::from)?
    }
}

/// Runs every migration the database has not seen yet, each in its own transaction
/// together with the version bump. A newer database is refused, never downgraded.
fn migrate(conn: &mut Connection) -> Result<(), DbError> {
    let version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let latest = i32::try_from(MIGRATIONS.len()).unwrap_or(i32::MAX);
    if version > latest {
        return Err(DbError::UnsupportedSchema(version));
    }
    let applied = usize::try_from(version).unwrap_or_default();
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(applied) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(
            None,
            "user_version",
            i32::try_from(index + 1).unwrap_or(i32::MAX),
        )?;
        tx.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persists_the_schema_version_and_reopens() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mail.db");
        drop(Database::open(&path).unwrap());
        let db = Database::open(&path).unwrap();
        let version: i32 = db
            .with(|conn| {
                conn.pragma_query_value(None, "user_version", |row| row.get(0))
                    .map_err(DbError::from)
            })
            .unwrap();
        assert_eq!(version, 3);
    }

    #[test]
    fn migrates_a_version_1_database() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mail.db");
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(MIGRATIONS[0]).unwrap();
            conn.pragma_update(None, "user_version", 1).unwrap();
        }
        let db = Database::open(&path).unwrap();
        let port: Option<u16> = db
            .with(|conn| {
                conn.query_row("SELECT smtp_port FROM settings", [], |row| row.get(0))
                    .map_err(DbError::from)
            })
            .unwrap();
        assert_eq!(port, None);
    }

    #[test]
    fn refuses_a_newer_database() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mail.db");
        Connection::open(&path)
            .unwrap()
            .pragma_update(None, "user_version", 99)
            .unwrap();
        assert!(matches!(
            Database::open(&path),
            Err(DbError::UnsupportedSchema(99))
        ));
    }
}
