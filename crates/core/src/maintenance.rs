//! Resetting Mailbeaver to a fresh database: every message, project and setting is
//! removed in one transaction while the schema stays (ADR-0018).

use crate::changes::{Change, ChangeFeed};
use crate::db::{Database, DbError};

/// Clears the data of every feature. The database file stays open; deleting it while
/// the servers run would break their connection and leave WAL files behind.
#[derive(Clone)]
pub struct MaintenanceService {
    db: Database,
    changes: ChangeFeed,
}

impl MaintenanceService {
    pub fn new(db: Database, changes: ChangeFeed) -> Self {
        Self { db, changes }
    }

    pub async fn reset(&self) -> Result<(), DbError> {
        self.db
            .run(|conn| {
                let tx = conn.transaction()?;
                tx.execute_batch(
                    "DELETE FROM messages;
                     DELETE FROM project_rules;
                     DELETE FROM projects;
                     DELETE FROM folder_views;
                     DELETE FROM logs;
                     DELETE FROM settings;
                     INSERT INTO settings (id) VALUES (1);
                     DELETE FROM sqlite_sequence WHERE name = 'messages';
                     INSERT INTO messages_fts (messages_fts) VALUES ('rebuild');",
                )?;
                tx.commit()?;
                // Gives the freed pages back to the file system; not allowed in a transaction.
                conn.execute_batch("VACUUM")?;
                Ok::<_, DbError>(())
            })
            .await?;
        self.changes.announce(Change::Reset);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::{Envelope, ListQuery, MessageId, MessageService};
    use crate::settings::{Grouping, Settings, SettingsService, TrashRetentionDays};

    #[tokio::test]
    async fn removes_everything_and_restarts_ids() {
        let db = Database::open_in_memory().unwrap();
        let changes = ChangeFeed::new();
        let messages =
            MessageService::new(db.clone(), changes.clone(), crate::stats::Stats::new(), 10);
        let settings = SettingsService::new(db.clone(), changes.clone());
        let auth = Envelope {
            auth_user: crate::messages::AuthUser::parse("shop"),
            ..Envelope::default()
        };
        messages
            .receive(auth, b"Subject: x\r\n\r\nbody\r\n".to_vec())
            .await
            .unwrap();
        settings
            .update(Settings {
                trash_retention_days: Some(TrashRetentionDays::try_from(7).unwrap()),
                tray_grouping: Grouping::Project,
                theme: crate::settings::Theme::Dark,
                sidebar_collapsed: true,
                notifications: crate::settings::Notifications::Off,
                notification_sound: true,
                unread_badge: false,
                update_check: true,
            })
            .await
            .unwrap();

        let mut feed = changes.subscribe();
        MaintenanceService::new(db.clone(), changes.clone())
            .reset()
            .await
            .unwrap();

        assert_eq!(feed.recv().await.unwrap(), Change::Reset);
        assert_eq!(messages.list(ListQuery::default()).await.unwrap().total, 0);
        assert_eq!(settings.get().await.unwrap(), Settings::default());
        let projects: i64 = db
            .with(|conn| {
                conn.query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))
                    .map_err(DbError::from)
            })
            .unwrap();
        assert_eq!(projects, 0);
        let id = messages
            .receive(Envelope::default(), b"Subject: y\r\n\r\nbody\r\n".to_vec())
            .await
            .unwrap();
        assert_eq!(id, MessageId(1));
    }
}
