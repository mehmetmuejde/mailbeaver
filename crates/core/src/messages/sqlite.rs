//! Storage of captured messages. Every function takes the connection (or transaction)
//! it works on, so services decide the transaction boundaries.

use std::collections::HashMap;

use rusqlite::types::Value;
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params, params_from_iter};

use crate::db::DbError;
use crate::projects::{ProjectId, ProjectRules};

use super::domain::{
    Counts, DeleteOutcome, Folder, FolderCounts, ListQuery, MessageId, MessagePage, MessageSummary,
    ProjectFilter, ProjectRef,
};
use super::search::{SearchProject, SearchQuery};

const SUMMARY_COLUMNS: &str = "m.id, m.received_at, m.header_from, m.header_to, m.subject, \
     m.size, m.has_attachments, m.is_read, m.folder, m.project_id, p.name AS project_name, \
     substr(m.body_text, 1, 400) AS preview_source";

/// Characters of body text shown in the message list.
const PREVIEW_CHARS: usize = 160;
const FROM_MESSAGES: &str = "FROM messages m LEFT JOIN projects p ON p.id = m.project_id";

// Addresses never contain line breaks, so a newline is a safe list separator.
const LIST_SEPARATOR: &str = "\n";

pub struct NewMessage<'a> {
    pub received_at: i64,
    pub envelope_from: &'a str,
    pub envelope_to: &'a [String],
    pub header_from: &'a str,
    pub header_to: &'a [String],
    pub sender_address: &'a str,
    pub auth_user: Option<&'a str>,
    pub subject: &'a str,
    pub body_text: &'a str,
    pub has_attachments: bool,
    pub project: Option<ProjectId>,
    pub raw: &'a [u8],
}

pub struct StoredMessage {
    pub summary: MessageSummary,
    pub envelope_from: String,
    pub envelope_to: Vec<String>,
    pub auth_user: Option<String>,
    pub raw: Vec<u8>,
}

/// Stores a message in the inbox and drops the oldest ones beyond `max_messages`.
/// Returns the new id and how many old messages were dropped.
pub fn insert(
    tx: &Transaction<'_>,
    message: &NewMessage<'_>,
    max_messages: u32,
) -> Result<(MessageId, usize), DbError> {
    tx.execute(
        "INSERT INTO messages (received_at, envelope_from, envelope_to, header_from, header_to,
            sender_address, auth_user, subject, body_text, size, has_attachments, project_id, raw)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            message.received_at,
            message.envelope_from,
            message.envelope_to.join(LIST_SEPARATOR),
            message.header_from,
            message.header_to.join(LIST_SEPARATOR),
            message.sender_address,
            message.auth_user,
            message.subject,
            message.body_text,
            i64::try_from(message.raw.len()).unwrap_or(i64::MAX),
            message.has_attachments,
            message.project.map(|project| project.0),
            message.raw,
        ],
    )?;
    let id = tx.last_insert_rowid();
    let dropped = tx.execute(
        "DELETE FROM messages WHERE id NOT IN
            (SELECT id FROM messages ORDER BY id DESC LIMIT ?1)",
        params![max_messages],
    )?;
    Ok((MessageId(id), dropped))
}

/// Newest first. Every filter is a fixed SQL fragment with a bound parameter.
pub fn list(conn: &Connection, query: &ListQuery) -> Result<MessagePage, DbError> {
    let mut conditions = Vec::new();
    let mut values = Vec::new();
    if let Some(folder) = query.folder {
        conditions.push("m.folder = ?");
        values.push(Value::from(folder.as_str().to_owned()));
    }
    if query.exclude_trash {
        conditions.push("m.folder != 'trash'");
    }
    match query.project {
        ProjectFilter::All => {}
        ProjectFilter::Unassigned => conditions.push("m.project_id IS NULL"),
        ProjectFilter::Project(project) => {
            conditions.push("m.project_id = ?");
            values.push(Value::from(project.0));
        }
    }
    if let Some(since) = query.since {
        conditions.push("m.received_at >= ?");
        values.push(Value::from(since));
    }
    if let Some(search) = &query.search {
        search_conditions(search, &mut conditions, &mut values);
    }
    let filter = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let (total, unread) = conn.query_row(
        &format!("SELECT COUNT(*), COALESCE(SUM(m.is_read = 0), 0) FROM messages m {filter}"),
        params_from_iter(&values),
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    values.push(Value::from(i64::from(query.limit)));
    values.push(Value::from(i64::from(query.offset)));
    let mut stmt = conn.prepare(&format!(
        "SELECT {SUMMARY_COLUMNS} {FROM_MESSAGES} {filter} ORDER BY m.id DESC LIMIT ? OFFSET ?"
    ))?;
    let messages = stmt
        .query_map(params_from_iter(&values), summary_from_row)?
        .collect::<Result<_, _>>()?;
    Ok(MessagePage {
        total,
        unread,
        messages,
    })
}

/// SQL for every part of a parsed search; the values are bound, never formatted in.
fn search_conditions(
    search: &SearchQuery,
    conditions: &mut Vec<&'static str>,
    values: &mut Vec<Value>,
) {
    if let Some(fts) = search.as_fts() {
        conditions.push("m.id IN (SELECT rowid FROM messages_fts WHERE messages_fts MATCH ?)");
        values.push(Value::from(fts.to_owned()));
    }
    if let Some(is_read) = search.is_read {
        conditions.push("m.is_read = ?");
        values.push(Value::from(is_read));
    }
    if let Some(has_attachments) = search.has_attachments {
        conditions.push("m.has_attachments = ?");
        values.push(Value::from(has_attachments));
    }
    match &search.project {
        None => {}
        Some(SearchProject::Unassigned) => conditions.push("m.project_id IS NULL"),
        Some(SearchProject::Named(name)) => {
            // `projects.name` compares without regard to case (COLLATE NOCASE).
            conditions.push("m.project_id IN (SELECT id FROM projects WHERE name = ?)");
            values.push(Value::from(name.clone()));
        }
    }
    if let Some(from) = search.received_from {
        conditions.push("m.received_at >= ?");
        values.push(Value::from(from));
    }
    if let Some(before) = search.received_before {
        conditions.push("m.received_at < ?");
        values.push(Value::from(before));
    }
}

pub fn folder_counts(conn: &Connection) -> Result<FolderCounts, DbError> {
    let mut stmt = conn.prepare(
        "SELECT folder, COUNT(*), COALESCE(SUM(is_read = 0), 0) FROM messages GROUP BY folder",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            Counts {
                total: row.get(1)?,
                unread: row.get(2)?,
            },
        ))
    })?;
    let mut counts = FolderCounts::default();
    for row in rows {
        let (folder, folder_counts) = row?;
        // The CHECK constraint allows only known folders.
        match folder.parse::<Folder>() {
            Ok(Folder::Inbox) => counts.inbox = folder_counts,
            Ok(Folder::Archive) => counts.archive = folder_counts,
            Ok(Folder::Trash) => counts.trash = folder_counts,
            Err(_) => {}
        }
    }
    Ok(counts)
}

/// Counts of the messages outside Trash per project; `None` collects the unassigned.
pub fn counts_by_project(conn: &Connection) -> Result<HashMap<Option<ProjectId>, Counts>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT project_id, COUNT(*), COALESCE(SUM(is_read = 0), 0)
         FROM messages WHERE folder != 'trash' GROUP BY project_id",
    )?;
    let counts = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, Option<i64>>(0)?.map(ProjectId),
                Counts {
                    total: row.get(1)?,
                    unread: row.get(2)?,
                },
            ))
        })?
        .collect::<Result<_, _>>()?;
    Ok(counts)
}

pub fn get(conn: &Connection, id: MessageId) -> Result<Option<StoredMessage>, DbError> {
    let message = conn
        .query_row(
            &format!(
                "SELECT {SUMMARY_COLUMNS}, m.envelope_from, m.envelope_to, m.auth_user, m.raw
                 {FROM_MESSAGES} WHERE m.id = ?1"
            ),
            params![id.0],
            |row| {
                let envelope_to: String = row.get("envelope_to")?;
                Ok(StoredMessage {
                    summary: summary_from_row(row)?,
                    envelope_from: row.get("envelope_from")?,
                    envelope_to: split_list(&envelope_to),
                    auth_user: row.get("auth_user")?,
                    raw: row.get("raw")?,
                })
            },
        )
        .optional()?;
    Ok(message)
}

pub fn raw(conn: &Connection, id: MessageId) -> Result<Option<Vec<u8>>, DbError> {
    let raw = conn
        .query_row(
            "SELECT raw FROM messages WHERE id = ?1",
            params![id.0],
            |row| row.get(0),
        )
        .optional()?;
    Ok(raw)
}

/// Returns `false` when the message does not exist.
pub fn mark_read(conn: &Connection, id: MessageId) -> Result<bool, DbError> {
    let changed = conn.execute(
        "UPDATE messages SET is_read = 1 WHERE id = ?1",
        params![id.0],
    )?;
    Ok(changed > 0)
}

/// Returns `false` when the message does not exist.
pub fn mark_unread(conn: &Connection, id: MessageId) -> Result<bool, DbError> {
    let changed = conn.execute(
        "UPDATE messages SET is_read = 0 WHERE id = ?1",
        params![id.0],
    )?;
    Ok(changed > 0)
}

/// `now` becomes the trash time when the message enters Trash. Returns `false` when the
/// message does not exist.
pub fn move_to(
    conn: &Connection,
    id: MessageId,
    folder: Folder,
    now: i64,
) -> Result<bool, DbError> {
    let changed = conn.execute(
        "UPDATE messages SET
            folder = ?2,
            trashed_at = CASE WHEN ?2 = 'trash' THEN COALESCE(trashed_at, ?3) END
         WHERE id = ?1",
        params![id.0, folder.as_str(), now],
    )?;
    Ok(changed > 0)
}

/// Deletes a message for good, but only from Trash.
pub fn delete(conn: &Connection, id: MessageId) -> Result<DeleteOutcome, DbError> {
    let changed = conn.execute(
        "DELETE FROM messages WHERE id = ?1 AND folder = 'trash'",
        params![id.0],
    )?;
    if changed > 0 {
        return Ok(DeleteOutcome::Deleted);
    }
    let exists: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM messages WHERE id = ?1)",
        params![id.0],
        |row| row.get(0),
    )?;
    Ok(if exists {
        DeleteOutcome::NotInTrash
    } else {
        DeleteOutcome::NotFound
    })
}

/// Marks every unread message of `folder` (all folders for `None`) read; returns how
/// many changed.
pub fn mark_all_read(conn: &Connection, folder: Option<Folder>) -> Result<usize, DbError> {
    Ok(conn.execute(
        "UPDATE messages SET is_read = 1 WHERE is_read = 0 AND (?1 IS NULL OR folder = ?1)",
        params![folder.map(Folder::as_str)],
    )?)
}

pub fn empty_trash(conn: &Connection) -> Result<usize, DbError> {
    Ok(conn.execute("DELETE FROM messages WHERE folder = 'trash'", [])?)
}

/// Deletes messages that entered Trash at or before `cutoff` (Unix milliseconds).
pub fn purge_trash(conn: &Connection, cutoff: i64) -> Result<usize, DbError> {
    Ok(conn.execute(
        "DELETE FROM messages WHERE folder = 'trash' AND trashed_at <= ?1",
        params![cutoff],
    )?)
}

/// Returns how many messages were deleted.
pub fn clear(conn: &Connection) -> Result<usize, DbError> {
    Ok(conn.execute("DELETE FROM messages", [])?)
}

/// Applies `rules` to every stored message and updates the ones whose project changes.
pub fn reassign_projects(tx: &Transaction<'_>, rules: &ProjectRules) -> Result<usize, DbError> {
    let messages = tx
        .prepare("SELECT id, sender_address, auth_user, project_id FROM messages")?
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<i64>>(3)?.map(ProjectId),
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut update = tx.prepare("UPDATE messages SET project_id = ?2 WHERE id = ?1")?;
    let mut changed = 0;
    for (id, sender, auth_user, current) in messages {
        let project = rules.resolve(&sender, auth_user.as_deref());
        if project != current {
            update.execute(params![id, project.map(|project| project.0)])?;
            changed += 1;
        }
    }
    Ok(changed)
}

fn summary_from_row(row: &Row<'_>) -> rusqlite::Result<MessageSummary> {
    let to: String = row.get("header_to")?;
    let folder: String = row.get("folder")?;
    let project_id: Option<i64> = row.get("project_id")?;
    let project_name: Option<String> = row.get("project_name")?;
    Ok(MessageSummary {
        id: MessageId(row.get("id")?),
        received_at: row.get("received_at")?,
        from: row.get("header_from")?,
        to: split_list(&to),
        subject: row.get("subject")?,
        size: row.get("size")?,
        has_attachments: row.get("has_attachments")?,
        is_read: row.get("is_read")?,
        folder: folder.parse().map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(err))
        })?,
        project: project_id.zip(project_name).map(|(id, name)| ProjectRef {
            id: ProjectId(id),
            name,
        }),
        preview: preview(&row.get::<_, String>("preview_source")?),
    })
}

/// Collapses whitespace and cuts the text to the preview length.
fn preview(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= PREVIEW_CHARS {
        return collapsed;
    }
    let cut: String = collapsed.chars().take(PREVIEW_CHARS).collect();
    format!("{}…", cut.trim_end())
}

fn split_list(value: &str) -> Vec<String> {
    value
        .split(LIST_SEPARATOR)
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::projects::ProjectRules;

    struct Mail<'a> {
        subject: &'a str,
        from: &'a str,
        body: &'a str,
    }

    const MAIL: Mail<'static> = Mail {
        subject: "subject",
        from: "a@example.com",
        body: "",
    };

    fn insert_into(conn: &mut Connection, mail: &Mail<'_>, max_messages: u32) -> MessageId {
        let tx = conn.transaction().unwrap();
        let (id, _) = insert(
            &tx,
            &NewMessage {
                received_at: 1,
                envelope_from: "a@example.com",
                envelope_to: &["b@example.com".into()],
                header_from: mail.from,
                header_to: &["b@example.com".into(), "c@example.com".into()],
                sender_address: "a@example.com",
                auth_user: None,
                subject: mail.subject,
                body_text: mail.body,
                has_attachments: false,
                project: None,
                raw: b"raw",
            },
            max_messages,
        )
        .unwrap();
        tx.commit().unwrap();
        id
    }

    fn with_db(test: impl FnOnce(&mut Connection)) {
        Database::open_in_memory()
            .unwrap()
            .with(|conn| {
                test(conn);
                Ok::<_, DbError>(())
            })
            .unwrap();
    }

    fn subjects(conn: &Connection, query: &ListQuery) -> Vec<String> {
        list(conn, query)
            .unwrap()
            .messages
            .into_iter()
            .map(|message| message.subject)
            .collect()
    }

    fn query() -> ListQuery {
        ListQuery {
            limit: 10,
            ..ListQuery::default()
        }
    }

    fn search(conn: &Connection, input: &str) -> Vec<String> {
        subjects(
            conn,
            &ListQuery {
                search: SearchQuery::parse(input),
                ..query()
            },
        )
    }

    #[test]
    fn lists_newest_first_with_counts() {
        with_db(|conn| {
            insert_into(
                conn,
                &Mail {
                    subject: "first",
                    ..MAIL
                },
                10,
            );
            let second = insert_into(
                conn,
                &Mail {
                    subject: "second",
                    ..MAIL
                },
                10,
            );
            mark_read(conn, second).unwrap();

            let page = list(conn, &query()).unwrap();
            assert_eq!((page.total, page.unread), (2, 1));
            assert_eq!(page.messages[0].subject, "second");
            assert!(page.messages[0].is_read);
            assert_eq!(page.messages[0].to, ["b@example.com", "c@example.com"]);
            assert_eq!(page.messages[0].folder, Folder::Inbox);
        });
    }

    #[test]
    fn retention_drops_oldest() {
        with_db(|conn| {
            let first = insert_into(conn, &MAIL, 2);
            insert_into(conn, &MAIL, 2);
            insert_into(conn, &MAIL, 2);
            assert_eq!(list(conn, &query()).unwrap().total, 2);
            assert!(get(conn, first).unwrap().is_none());
        });
    }

    #[test]
    fn filters_by_folder_and_counts_per_folder() {
        with_db(|conn| {
            let archived = insert_into(
                conn,
                &Mail {
                    subject: "archived",
                    ..MAIL
                },
                10,
            );
            insert_into(
                conn,
                &Mail {
                    subject: "kept",
                    ..MAIL
                },
                10,
            );
            assert!(move_to(conn, archived, Folder::Archive, 5).unwrap());
            assert!(!move_to(conn, MessageId(999), Folder::Archive, 5).unwrap());

            let inbox = ListQuery {
                folder: Some(Folder::Inbox),
                ..query()
            };
            assert_eq!(subjects(conn, &inbox), ["kept"]);
            let counts = folder_counts(conn).unwrap();
            assert_eq!(
                counts.inbox,
                Counts {
                    total: 1,
                    unread: 1
                }
            );
            assert_eq!(
                counts.archive,
                Counts {
                    total: 1,
                    unread: 1
                }
            );
            assert_eq!(counts.trash, Counts::default());
        });
    }

    #[test]
    fn deletes_permanently_only_from_trash() {
        with_db(|conn| {
            let id = insert_into(conn, &MAIL, 10);
            assert_eq!(delete(conn, id).unwrap(), DeleteOutcome::NotInTrash);
            move_to(conn, id, Folder::Trash, 5).unwrap();
            assert_eq!(delete(conn, id).unwrap(), DeleteOutcome::Deleted);
            assert_eq!(delete(conn, id).unwrap(), DeleteOutcome::NotFound);
        });
    }

    #[test]
    fn restoring_clears_the_trash_time() {
        with_db(|conn| {
            let id = insert_into(conn, &MAIL, 10);
            move_to(conn, id, Folder::Trash, 5).unwrap();
            move_to(conn, id, Folder::Inbox, 6).unwrap();
            move_to(conn, id, Folder::Trash, 50).unwrap();
            // Trashed again at 50, so a cutoff before that keeps it.
            assert_eq!(purge_trash(conn, 10).unwrap(), 0);
            assert_eq!(purge_trash(conn, 50).unwrap(), 1);
        });
    }

    #[test]
    fn empties_trash_and_clears_everything() {
        with_db(|conn| {
            let trashed = insert_into(
                conn,
                &Mail {
                    subject: "trashed",
                    ..MAIL
                },
                10,
            );
            insert_into(
                conn,
                &Mail {
                    subject: "kept",
                    ..MAIL
                },
                10,
            );
            move_to(conn, trashed, Folder::Trash, 5).unwrap();
            assert_eq!(empty_trash(conn).unwrap(), 1);
            assert_eq!(subjects(conn, &query()), ["kept"]);
            clear(conn).unwrap();
            assert_eq!(list(conn, &query()).unwrap().total, 0);
        });
    }

    #[test]
    fn searches_subject_sender_and_body_by_prefix() {
        with_db(|conn| {
            insert_into(
                conn,
                &Mail {
                    subject: "Invoice March",
                    body: "Your total is 42 EUR",
                    ..MAIL
                },
                10,
            );
            insert_into(
                conn,
                &Mail {
                    subject: "Password reset",
                    from: "Support <support@shop.example>",
                    ..MAIL
                },
                10,
            );
            assert_eq!(search(conn, "invo"), ["Invoice March"]);
            assert_eq!(search(conn, "total 42"), ["Invoice March"]);
            assert_eq!(search(conn, "support@shop"), ["Password reset"]);
            assert!(search(conn, "invoice reset").is_empty());
            assert!(search(conn, "a* OR -b").is_empty());
        });
    }

    #[test]
    fn applies_the_search_filters() {
        with_db(|conn| {
            let order = insert_into(
                conn,
                &Mail {
                    subject: "Order shipped",
                    from: "Shop <orders@shop.example>",
                    body: "Your parcel from the blog",
                },
                10,
            );
            let news = insert_into(
                conn,
                &Mail {
                    subject: "Weekly news from the shop",
                    from: "Blog <news@blog.example>",
                    ..MAIL
                },
                10,
            );
            conn.execute_batch(
                "INSERT INTO projects (id, name, created_at) VALUES (1, 'Shop', 0);",
            )
            .unwrap();
            conn.execute(
                "UPDATE messages SET project_id = 1, has_attachments = 1, received_at = ?2
                 WHERE id = ?1",
                params![order.0, 1_790_812_800_000_i64],
            )
            .unwrap();
            conn.execute(
                "UPDATE messages SET is_read = 1, received_at = ?2 WHERE id = ?1",
                params![news.0, 1_792_195_200_000_i64],
            )
            .unwrap();

            assert_eq!(search(conn, "from:shop"), ["Order shipped"]);
            assert_eq!(search(conn, "subject:shop"), ["Weekly news from the shop"]);
            assert_eq!(search(conn, "body:blog"), ["Order shipped"]);
            assert_eq!(search(conn, "is:unread"), ["Order shipped"]);
            assert_eq!(search(conn, "is:read news"), ["Weekly news from the shop"]);
            assert_eq!(search(conn, "has:attachment"), ["Order shipped"]);
            assert_eq!(search(conn, "project:shop"), ["Order shipped"]);
            assert_eq!(search(conn, "project:none"), ["Weekly news from the shop"]);
            assert_eq!(
                search(conn, "after:2026-10-17"),
                ["Weekly news from the shop"]
            );
            assert_eq!(search(conn, "before:2026-10-17"), ["Order shipped"]);
            assert!(search(conn, "project:unknown").is_empty());
            assert!(search(conn, "from:blog has:attachment").is_empty());
        });
    }

    #[test]
    fn previews_the_body_and_filters_by_time() {
        with_db(|conn| {
            insert_into(
                conn,
                &Mail {
                    subject: "old",
                    body: "Hello\n\n   there,   friend",
                    ..MAIL
                },
                10,
            );
            conn.execute("UPDATE messages SET received_at = 500", [])
                .unwrap();
            let long = "x".repeat(300);
            insert_into(
                conn,
                &Mail {
                    subject: "new",
                    body: &long,
                    ..MAIL
                },
                10,
            );
            conn.execute(
                "UPDATE messages SET received_at = 2000 WHERE subject = 'new'",
                [],
            )
            .unwrap();

            let page = list(conn, &query()).unwrap();
            assert_eq!(page.messages[1].preview, "Hello there, friend");
            assert_eq!(page.messages[0].preview.chars().count(), PREVIEW_CHARS + 1);
            let recent = ListQuery {
                since: Some(1000),
                ..query()
            };
            assert_eq!(subjects(conn, &recent), ["new"]);
        });
    }

    #[test]
    fn marks_all_messages_of_a_folder_read() {
        with_db(|conn| {
            let archived = insert_into(conn, &MAIL, 10);
            insert_into(conn, &MAIL, 10);
            move_to(conn, archived, Folder::Archive, 1).unwrap();
            assert_eq!(mark_all_read(conn, Some(Folder::Inbox)).unwrap(), 1);
            assert_eq!(folder_counts(conn).unwrap().archive.unread, 1);
            assert_eq!(mark_all_read(conn, None).unwrap(), 1);
            assert_eq!(mark_all_read(conn, None).unwrap(), 0);
        });
    }

    #[test]
    fn search_index_follows_deletes() {
        with_db(|conn| {
            let id = insert_into(
                conn,
                &Mail {
                    subject: "Unique subject",
                    ..MAIL
                },
                10,
            );
            move_to(conn, id, Folder::Trash, 1).unwrap();
            empty_trash(conn).unwrap();
            assert!(search(conn, "unique").is_empty());
        });
    }

    #[test]
    fn reassigns_projects_and_filters_by_them() {
        with_db(|conn| {
            conn.execute(
                "INSERT INTO projects (id, name, created_at) VALUES (1, 'Shop', 0)",
                [],
            )
            .unwrap();
            insert_into(conn, &MAIL, 10);
            let rules = ProjectRules::new(vec![(
                ProjectId(1),
                crate::projects::Rule::parse(
                    crate::projects::RuleKind::SenderDomain,
                    "example.com",
                )
                .unwrap(),
            )]);
            let tx = conn.transaction().unwrap();
            assert_eq!(reassign_projects(&tx, &rules).unwrap(), 1);
            assert_eq!(reassign_projects(&tx, &rules).unwrap(), 0);
            tx.commit().unwrap();

            let shop = ListQuery {
                project: ProjectFilter::Project(ProjectId(1)),
                ..query()
            };
            assert_eq!(list(conn, &shop).unwrap().total, 1);
            let unassigned = ListQuery {
                project: ProjectFilter::Unassigned,
                ..query()
            };
            assert_eq!(list(conn, &unassigned).unwrap().total, 0);
            let summary = &list(conn, &query()).unwrap().messages[0];
            assert_eq!(
                summary.project.as_ref().map(|p| p.name.as_str()),
                Some("Shop")
            );
            assert_eq!(
                counts_by_project(conn).unwrap()[&Some(ProjectId(1))].total,
                1
            );
        });
    }
}
