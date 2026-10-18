//! Storage of projects and their rules.

use std::collections::BTreeMap;

use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::db::DbError;

use super::domain::{
    Project, ProjectId, ProjectName, ProjectRules, Rule, RuleKind, RuleSuggestion,
};

pub fn load_rules(conn: &Connection) -> Result<ProjectRules, DbError> {
    let mut stmt = conn.prepare("SELECT project_id, kind, value FROM project_rules")?;
    let rules = stmt
        .query_map([], |row| {
            Ok((
                ProjectId(row.get(0)?),
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        // The CHECK constraint keeps unknown kinds out; skipping one is the safe choice.
        .filter_map(|(id, kind, value)| {
            Some((
                id,
                Rule {
                    kind: RuleKind::from_db(&kind)?,
                    value,
                },
            ))
        })
        .collect();
    Ok(ProjectRules::new(rules))
}

pub fn list(conn: &Connection) -> Result<Vec<Project>, DbError> {
    let mut rules: BTreeMap<i64, Vec<Rule>> = BTreeMap::new();
    let mut stmt = conn.prepare("SELECT project_id, kind, value FROM project_rules ORDER BY id")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    for row in rows {
        let (project, kind, value) = row?;
        if let Some(kind) = RuleKind::from_db(&kind) {
            rules.entry(project).or_default().push(Rule { kind, value });
        }
    }

    let mut stmt = conn.prepare("SELECT id, name, notify FROM projects ORDER BY name")?;
    let projects = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, bool>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter_map(|(id, name, notify)| {
            Some(Project {
                id: ProjectId(id),
                name: ProjectName::parse(&name).ok()?,
                rules: rules.remove(&id).unwrap_or_default(),
                notify,
            })
        })
        .collect();
    Ok(projects)
}

pub fn find(conn: &Connection, id: ProjectId) -> Result<Option<Project>, DbError> {
    Ok(list(conn)?.into_iter().find(|project| project.id == id))
}

/// The project that already uses `name` (case-insensitive), if any.
pub fn id_by_name(conn: &Connection, name: &ProjectName) -> Result<Option<ProjectId>, DbError> {
    let id = conn
        .query_row(
            "SELECT id FROM projects WHERE name = ?1",
            params![name.as_str()],
            |row| row.get(0),
        )
        .optional()?;
    Ok(id.map(ProjectId))
}

/// The project that already owns `rule`, if any.
pub fn rule_owner(conn: &Connection, rule: &Rule) -> Result<Option<ProjectId>, DbError> {
    let id = conn
        .query_row(
            "SELECT project_id FROM project_rules WHERE kind = ?1 AND value = ?2",
            params![rule.kind.as_str(), rule.value],
            |row| row.get(0),
        )
        .optional()?;
    Ok(id.map(ProjectId))
}

pub fn insert(
    tx: &Transaction<'_>,
    name: &ProjectName,
    notify: bool,
    now: i64,
) -> Result<ProjectId, DbError> {
    tx.execute(
        "INSERT INTO projects (name, notify, created_at) VALUES (?1, ?2, ?3)",
        params![name.as_str(), notify, now],
    )?;
    Ok(ProjectId(tx.last_insert_rowid()))
}

/// Returns `false` when the project does not exist.
pub fn update(
    tx: &Transaction<'_>,
    id: ProjectId,
    name: &ProjectName,
    notify: bool,
) -> Result<bool, DbError> {
    let changed = tx.execute(
        "UPDATE projects SET name = ?2, notify = ?3 WHERE id = ?1",
        params![id.0, name.as_str(), notify],
    )?;
    Ok(changed > 0)
}

pub fn replace_rules(tx: &Transaction<'_>, id: ProjectId, rules: &[Rule]) -> Result<(), DbError> {
    tx.execute(
        "DELETE FROM project_rules WHERE project_id = ?1",
        params![id.0],
    )?;
    let mut stmt =
        tx.prepare("INSERT INTO project_rules (project_id, kind, value) VALUES (?1, ?2, ?3)")?;
    for rule in rules {
        stmt.execute(params![id.0, rule.kind.as_str(), rule.value])?;
    }
    Ok(())
}

const SUGGESTION_LIMIT: i64 = 20;

/// The most frequent senders, sender domains or SMTP users of mail outside Trash that
/// start with `prefix` and no rule of that kind names yet.
pub fn rule_suggestions(
    conn: &Connection,
    kind: RuleKind,
    prefix: &str,
) -> Result<Vec<RuleSuggestion>, DbError> {
    let pattern = format!(
        "{}%",
        prefix
            .to_lowercase()
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    );
    let mut stmt = conn.prepare(
        "SELECT v.value, COUNT(*) AS n FROM (
             SELECT CASE ?1
                 WHEN 'sender_address' THEN sender_address
                 WHEN 'sender_domain' THEN
                     CASE WHEN instr(sender_address, '@') > 0
                     THEN substr(sender_address, instr(sender_address, '@') + 1) END
                 ELSE lower(trim(auth_user))
             END AS value
             FROM messages WHERE folder != 'trash'
         ) AS v
         WHERE v.value IS NOT NULL AND v.value != '' AND v.value LIKE ?2 ESCAPE '\\'
           AND NOT EXISTS (
               SELECT 1 FROM project_rules r WHERE r.kind = ?1 AND r.value = v.value
           )
         GROUP BY v.value ORDER BY n DESC, v.value LIMIT ?3",
    )?;
    let suggestions = stmt
        .query_map(params![kind.as_str(), pattern, SUGGESTION_LIMIT], |row| {
            Ok(RuleSuggestion {
                value: row.get(0)?,
                count: row.get(1)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(suggestions)
}

/// Returns `false` when the project does not exist. Its messages lose their project.
pub fn delete(tx: &Transaction<'_>, id: ProjectId) -> Result<bool, DbError> {
    let changed = tx.execute("DELETE FROM projects WHERE id = ?1", params![id.0])?;
    Ok(changed > 0)
}
