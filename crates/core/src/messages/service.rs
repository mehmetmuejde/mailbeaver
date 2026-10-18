//! Use cases for captured mail, shared by every frontend (SMTP, HTTP API, tray).

use crate::changes::{Change, ChangeFeed};
use crate::clock::now_millis;
use crate::db::{Database, DbError};
use crate::projects;
use crate::stats::Stats;

use super::domain::{
    DeleteOutcome, Envelope, Folder, FolderCounts, ListQuery, MessageDetail, MessageId, MessagePage,
};
use super::mime::{self, Attachment};
use super::sqlite::{self, NewMessage};

#[derive(Clone)]
pub struct MessageService {
    db: Database,
    changes: ChangeFeed,
    stats: Stats,
    /// Retention limit: the oldest messages beyond it are dropped (ADR-0004).
    max_messages: u32,
}

impl MessageService {
    pub fn new(db: Database, changes: ChangeFeed, stats: Stats, max_messages: u32) -> Self {
        Self {
            db,
            changes,
            stats,
            max_messages,
        }
    }

    /// Counters the SMTP server adds its rejections to.
    pub fn stats(&self) -> &Stats {
        &self.stats
    }

    /// Stores a message and assigns its project: a rule decides, otherwise an SMTP user
    /// no rule knows gets a project of its own.
    pub async fn receive(&self, envelope: Envelope, raw: Vec<u8>) -> Result<MessageId, DbError> {
        let max_messages = self.max_messages;
        let size = raw.len();
        let (id, dropped, created_project) = self
            .db
            .run(move |conn| {
                let summary = mime::summarize(&raw);
                let sender = if summary.from_address.is_empty() {
                    envelope.mail_from.to_lowercase()
                } else {
                    summary.from_address.to_lowercase()
                };
                let auth_user = envelope.auth_user.as_ref().map(|user| user.as_str());
                let now = now_millis();

                let tx = conn.transaction()?;
                let mut project = projects::load_rules(&tx)?.resolve(&sender, auth_user);
                let mut created_project = false;
                if let (None, Some(user)) = (project, auth_user) {
                    project = projects::project_for_new_auth_user(&tx, user, now)?;
                    created_project = project.is_some();
                }
                let (id, dropped) = sqlite::insert(
                    &tx,
                    &NewMessage {
                        received_at: now,
                        envelope_from: &envelope.mail_from,
                        envelope_to: &envelope.rcpt_to,
                        header_from: &summary.from,
                        header_to: &summary.to,
                        sender_address: &sender,
                        auth_user,
                        subject: &summary.subject,
                        body_text: &summary.body_text,
                        has_attachments: summary.has_attachments,
                        project,
                        raw: &raw,
                    },
                    max_messages,
                )?;
                tx.commit()?;
                Ok::<_, DbError>((id, dropped, created_project))
            })
            .await?;
        self.stats.record_accepted(size);
        self.stats.record_deleted(dropped);
        if created_project {
            self.changes.announce(Change::ProjectsChanged);
        }
        self.changes.announce(Change::Received { id });
        Ok(id)
    }

    pub async fn list(&self, query: ListQuery) -> Result<MessagePage, DbError> {
        self.db.run(move |conn| sqlite::list(conn, &query)).await
    }

    pub async fn folder_counts(&self) -> Result<FolderCounts, DbError> {
        self.db.run(|conn| sqlite::folder_counts(conn)).await
    }

    pub async fn detail(&self, id: MessageId) -> Result<Option<MessageDetail>, DbError> {
        self.db
            .run(move |conn| {
                Ok(sqlite::get(conn, id)?.map(|stored| MessageDetail {
                    body: mime::parse_body(&stored.raw),
                    summary: stored.summary,
                    envelope_from: stored.envelope_from,
                    envelope_to: stored.envelope_to,
                    auth_user: stored.auth_user,
                }))
            })
            .await
    }

    pub async fn raw(&self, id: MessageId) -> Result<Option<Vec<u8>>, DbError> {
        self.db.run(move |conn| sqlite::raw(conn, id)).await
    }

    pub async fn attachment(
        &self,
        id: MessageId,
        index: usize,
    ) -> Result<Option<Attachment>, DbError> {
        self.db
            .run(move |conn| {
                Ok(sqlite::raw(conn, id)?.and_then(|raw| mime::attachment(&raw, index)))
            })
            .await
    }

    /// Returns `false` when the message does not exist.
    pub async fn mark_read(&self, id: MessageId) -> Result<bool, DbError> {
        let found = self.db.run(move |conn| sqlite::mark_read(conn, id)).await?;
        if found {
            self.changes.announce(Change::Read { id });
        }
        Ok(found)
    }

    /// Returns `false` when the message does not exist.
    pub async fn mark_unread(&self, id: MessageId) -> Result<bool, DbError> {
        let found = self
            .db
            .run(move |conn| sqlite::mark_unread(conn, id))
            .await?;
        if found {
            self.changes.announce(Change::Unread { id });
        }
        Ok(found)
    }

    /// Returns `false` when the message does not exist.
    pub async fn move_to(&self, id: MessageId, folder: Folder) -> Result<bool, DbError> {
        let found = self
            .db
            .run(move |conn| sqlite::move_to(conn, id, folder, now_millis()))
            .await?;
        if found {
            self.changes.announce(Change::Moved { id, folder });
        }
        Ok(found)
    }

    /// Deletes a message for good; only messages in Trash can be deleted.
    pub async fn delete(&self, id: MessageId) -> Result<DeleteOutcome, DbError> {
        let outcome = self.db.run(move |conn| sqlite::delete(conn, id)).await?;
        if outcome == DeleteOutcome::Deleted {
            self.stats.record_deleted(1);
            self.changes.announce(Change::Deleted { id });
        }
        Ok(outcome)
    }

    pub async fn empty_trash(&self) -> Result<usize, DbError> {
        let count = self.db.run(|conn| sqlite::empty_trash(conn)).await?;
        self.stats.record_deleted(count);
        self.changes.announce(Change::TrashEmptied { count });
        Ok(count)
    }

    /// Deletes messages that entered Trash at or before `cutoff` (Unix milliseconds).
    pub async fn purge_trash(&self, cutoff: i64) -> Result<usize, DbError> {
        let count = self
            .db
            .run(move |conn| sqlite::purge_trash(conn, cutoff))
            .await?;
        if count > 0 {
            self.stats.record_deleted(count);
            self.changes.announce(Change::TrashEmptied { count });
        }
        Ok(count)
    }

    pub async fn clear(&self) -> Result<(), DbError> {
        let count = self.db.run(|conn| sqlite::clear(conn)).await?;
        self.stats.record_deleted(count);
        self.changes.announce(Change::Cleared);
        Ok(())
    }

    /// Marks every unread message of `folder` (all folders for `None`) read.
    pub async fn mark_all_read(&self, folder: Option<Folder>) -> Result<usize, DbError> {
        let count = self
            .db
            .run(move |conn| sqlite::mark_all_read(conn, folder))
            .await?;
        if count > 0 {
            self.changes.announce(Change::MarkedRead { count });
        }
        Ok(count)
    }
}
