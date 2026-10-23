//! Storage of the log entries.

use rusqlite::types::Value;
use rusqlite::{Connection, params, params_from_iter};

use crate::db::DbError;

use super::domain::{Level, LogEntry, LogQuery, NewLogEntry, Source};

pub fn insert(conn: &mut Connection, entries: &[NewLogEntry], keep: u32) -> Result<(), DbError> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO logs (created_at, level, source, target, message)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        for entry in entries {
            stmt.execute(params![
                entry.created_at,
                entry.level.as_str(),
                entry.source.as_str(),
                entry.target,
                entry.message,
            ])?;
        }
    }
    // Ids only grow, so everything below the newest `keep` goes.
    tx.execute(
        "DELETE FROM logs WHERE id <= (SELECT MAX(id) FROM logs) - ?1",
        params![keep],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn list(conn: &Connection, query: &LogQuery) -> Result<Vec<LogEntry>, DbError> {
    let levels = query.min_level.and_more_severe();
    let mut values: Vec<Value> = levels
        .iter()
        .map(|level| Value::from(level.as_str().to_owned()))
        .collect();
    let placeholders = vec!["?"; levels.len()].join(", ");
    let mut sql = format!(
        "SELECT id, created_at, level, source, target, message FROM logs
         WHERE level IN ({placeholders})"
    );
    if let Some(source) = query.source {
        sql.push_str(" AND source = ?");
        values.push(Value::from(source.as_str().to_owned()));
    }
    sql.push_str(" ORDER BY id DESC LIMIT ?");
    values.push(Value::from(i64::from(query.limit)));
    let mut stmt = conn.prepare(&sql)?;
    let entries = stmt
        .query_map(params_from_iter(&values), |row| {
            Ok(LogEntry {
                id: row.get(0)?,
                created_at: row.get(1)?,
                level: Level::from_db(&row.get::<_, String>(2)?),
                source: Source::from_db(&row.get::<_, String>(3)?),
                target: row.get(4)?,
                message: row.get(5)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(entries)
}

pub fn clear(conn: &Connection) -> Result<usize, DbError> {
    Ok(conn.execute("DELETE FROM logs", [])?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn messages(conn: &Connection, query: &LogQuery) -> Vec<String> {
        list(conn, query)
            .unwrap()
            .into_iter()
            .map(|entry| entry.message)
            .collect()
    }

    fn entry(level: Level, source: Source, message: &str) -> NewLogEntry {
        NewLogEntry::new(1, level, source, "test", message)
    }

    #[test]
    fn stores_filters_and_keeps_only_the_newest() {
        Database::open_in_memory()
            .unwrap()
            .with(|conn| {
                insert(
                    conn,
                    &[
                        entry(Level::Info, Source::Backend, "one"),
                        entry(Level::Warn, Source::Backend, "two"),
                        entry(Level::Error, Source::Frontend, "three"),
                    ],
                    10,
                )?;
                let all = LogQuery {
                    min_level: Level::Info,
                    source: None,
                    limit: 10,
                };
                assert_eq!(messages(conn, &all), ["three", "two", "one"]);
                assert_eq!(
                    messages(
                        conn,
                        &LogQuery {
                            min_level: Level::Warn,
                            ..all
                        }
                    ),
                    ["three", "two"]
                );
                assert_eq!(
                    messages(
                        conn,
                        &LogQuery {
                            source: Some(Source::Backend),
                            ..all
                        }
                    ),
                    ["two", "one"]
                );

                insert(conn, &[entry(Level::Info, Source::Backend, "four")], 2)?;
                assert_eq!(messages(conn, &all), ["four", "three"]);
                assert_eq!(clear(conn)?, 2);
                assert!(messages(conn, &all).is_empty());
                Ok::<_, DbError>(())
            })
            .unwrap();
    }
}
