//! Empties expired messages from Trash: hourly, and right after the retention setting
//! changes (ADR-0008).

use std::time::Duration;

use tokio::sync::broadcast::error::RecvError;
use tracing::{info, warn};

use crate::changes::{Change, ChangeFeed};
use crate::clock::now_millis;
use crate::settings::SettingsService;

use super::service::MessageService;

/// How often expired messages are removed from Trash.
pub const TRASH_CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60);

const MILLIS_PER_DAY: i64 = 24 * 60 * 60 * 1000;

/// Runs for the lifetime of the process.
pub async fn auto_empty_trash(
    messages: MessageService,
    settings: SettingsService,
    changes: ChangeFeed,
    interval: Duration,
) {
    let mut ticks = tokio::time::interval(interval);
    ticks.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut feed = changes.subscribe();
    loop {
        tokio::select! {
            _ = ticks.tick() => {}
            change = feed.recv() => match change {
                Ok(Change::SettingsChanged) | Err(RecvError::Lagged(_)) => {}
                Ok(_) => continue,
                Err(RecvError::Closed) => return,
            },
        }
        match purge_expired(&messages, &settings, now_millis()).await {
            Ok(0) => {}
            Ok(count) => info!(count, "expired messages removed from trash"),
            Err(err) => warn!(error = %err, "emptying expired trash failed"),
        }
    }
}

/// Deletes messages that have been in Trash longer than the configured retention.
/// Nothing happens while the retention is off.
pub async fn purge_expired(
    messages: &MessageService,
    settings: &SettingsService,
    now: i64,
) -> Result<usize, crate::db::DbError> {
    let Some(days) = settings.get().await?.trash_retention_days else {
        return Ok(0);
    };
    let cutoff = now.saturating_sub(i64::from(days.get()) * MILLIS_PER_DAY);
    messages.purge_trash(cutoff).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::messages::{Envelope, Folder};
    use crate::settings::{Grouping, Settings, TrashRetentionDays};

    #[tokio::test]
    async fn purges_trash_only_after_the_retention() {
        let db = Database::open_in_memory().unwrap();
        let changes = ChangeFeed::new();
        let messages =
            MessageService::new(db.clone(), changes.clone(), crate::stats::Stats::new(), 10);
        let settings = SettingsService::new(db, changes.clone());
        let id = messages
            .receive(Envelope::default(), b"Subject: x\r\n\r\nbody\r\n".to_vec())
            .await
            .unwrap();
        messages.move_to(id, Folder::Trash).await.unwrap();
        let now = now_millis();

        // Off by default: nothing is purged however old.
        assert_eq!(
            purge_expired(&messages, &settings, now + 400 * MILLIS_PER_DAY)
                .await
                .unwrap(),
            0
        );

        settings
            .update(Settings {
                trash_retention_days: Some(TrashRetentionDays::try_from(7).unwrap()),
                tray_grouping: Grouping::All,
                ..Settings::default()
            })
            .await
            .unwrap();
        assert_eq!(
            purge_expired(&messages, &settings, now + MILLIS_PER_DAY)
                .await
                .unwrap(),
            0
        );
        let mut feed = changes.subscribe();
        assert_eq!(
            purge_expired(&messages, &settings, now + 8 * MILLIS_PER_DAY)
                .await
                .unwrap(),
            1
        );
        assert_eq!(
            feed.recv().await.unwrap(),
            Change::TrashEmptied { count: 1 }
        );
    }
}
