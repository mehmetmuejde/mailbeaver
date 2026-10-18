//! Use cases for projects. Every change re-assigns the existing messages, so the rules
//! always describe the current state.

use std::collections::BTreeSet;

use rusqlite::{Connection, Transaction};

use crate::changes::{Change, ChangeFeed};
use crate::db::{Database, DbError};
use crate::messages::{self, Counts};

use super::domain::{
    MAX_RULES_PER_PROJECT, Project, ProjectId, ProjectName, Rule, RuleKind, RuleSuggestion,
};
use super::sqlite;
use utoipa::ToSchema;

#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("project not found")]
    NotFound,
    #[error("a project named {0:?} already exists")]
    NameTaken(String),
    #[error("{value:?} already belongs to project {project:?}")]
    RuleTaken { value: String, project: String },
    #[error("a project can have at most {MAX_RULES_PER_PROJECT} rules")]
    TooManyRules,
    #[error(transparent)]
    Db(#[from] DbError),
}

/// What a project consists of when it is created or changed.
#[derive(Debug, Clone)]
pub struct ProjectInput {
    pub name: ProjectName,
    pub rules: Vec<Rule>,
    pub notify: bool,
}

#[derive(Debug, Clone, serde::Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWithCounts {
    #[serde(flatten)]
    pub project: Project,
    #[serde(flatten)]
    pub counts: Counts,
}

/// Projects with the counts of their messages outside Trash, plus the messages that
/// belong to no project.
#[derive(Debug, Clone, serde::Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectOverview {
    pub projects: Vec<ProjectWithCounts>,
    pub unassigned: Counts,
}

#[derive(Clone)]
pub struct ProjectService {
    db: Database,
    changes: ChangeFeed,
}

impl ProjectService {
    pub fn new(db: Database, changes: ChangeFeed) -> Self {
        Self { db, changes }
    }

    pub async fn overview(&self) -> Result<ProjectOverview, ProjectError> {
        self.db.run(|conn| Ok(overview(conn)?)).await
    }

    pub async fn rule_suggestions(
        &self,
        kind: RuleKind,
        prefix: String,
    ) -> Result<Vec<RuleSuggestion>, DbError> {
        self.db
            .run(move |conn| sqlite::rule_suggestions(conn, kind, &prefix))
            .await
    }

    pub async fn create(&self, input: ProjectInput, now: i64) -> Result<Project, ProjectError> {
        self.write(move |tx| {
            check_free(tx, None, &input)?;
            let id = sqlite::insert(tx, &input.name, input.notify, now)?;
            sqlite::replace_rules(tx, id, &input.rules)?;
            Ok(id)
        })
        .await
    }

    pub async fn update(
        &self,
        id: ProjectId,
        input: ProjectInput,
    ) -> Result<Project, ProjectError> {
        self.write(move |tx| {
            check_free(tx, Some(id), &input)?;
            if !sqlite::update(tx, id, &input.name, input.notify)? {
                return Err(ProjectError::NotFound);
            }
            sqlite::replace_rules(tx, id, &input.rules)?;
            Ok(id)
        })
        .await
    }

    /// Deletes a project; its messages are re-assigned by the remaining rules.
    pub async fn delete(&self, id: ProjectId) -> Result<(), ProjectError> {
        self.db
            .run(move |conn| {
                let tx = conn.transaction().map_err(DbError::from)?;
                if !sqlite::delete(&tx, id)? {
                    return Err(ProjectError::NotFound);
                }
                messages::reassign_projects(&tx, &sqlite::load_rules(&tx)?)?;
                tx.commit().map_err(DbError::from)?;
                Ok::<_, ProjectError>(())
            })
            .await?;
        self.changes.announce(Change::ProjectsChanged);
        Ok(())
    }

    /// Runs a create or update in one transaction, re-assigns all messages and returns
    /// the stored project.
    async fn write<F>(&self, work: F) -> Result<Project, ProjectError>
    where
        F: FnOnce(&Transaction<'_>) -> Result<ProjectId, ProjectError> + Send + 'static,
    {
        let project = self
            .db
            .run(move |conn| {
                let tx = conn.transaction().map_err(DbError::from)?;
                let id = work(&tx)?;
                messages::reassign_projects(&tx, &sqlite::load_rules(&tx)?)?;
                let project = sqlite::find(&tx, id)?.ok_or(ProjectError::NotFound)?;
                tx.commit().map_err(DbError::from)?;
                Ok::<_, ProjectError>(project)
            })
            .await?;
        self.changes.announce(Change::ProjectsChanged);
        Ok(project)
    }
}

fn overview(conn: &Connection) -> Result<ProjectOverview, DbError> {
    let mut counts = messages::counts_by_project(conn)?;
    let projects = sqlite::list(conn)?
        .into_iter()
        .map(|project| ProjectWithCounts {
            counts: counts.remove(&Some(project.id)).unwrap_or_default(),
            project,
        })
        .collect();
    Ok(ProjectOverview {
        projects,
        unassigned: counts.remove(&None).unwrap_or_default(),
    })
}

/// A project name and every rule may belong to one project only.
fn check_free(
    tx: &Transaction<'_>,
    own: Option<ProjectId>,
    input: &ProjectInput,
) -> Result<(), ProjectError> {
    if input.rules.len() > MAX_RULES_PER_PROJECT {
        return Err(ProjectError::TooManyRules);
    }
    if let Some(other) = sqlite::id_by_name(tx, &input.name)?
        && Some(other) != own
    {
        return Err(ProjectError::NameTaken(input.name.as_str().to_owned()));
    }
    for rule in &input.rules {
        if let Some(other) = sqlite::rule_owner(tx, rule)?
            && Some(other) != own
        {
            let project = sqlite::find(tx, other)?
                .map(|project| project.name.as_str().to_owned())
                .unwrap_or_default();
            return Err(ProjectError::RuleTaken {
                value: rule.value.clone(),
                project,
            });
        }
    }
    Ok(())
}

/// Removes duplicate rules and keeps a stable order.
pub fn dedupe_rules(rules: Vec<Rule>) -> Vec<Rule> {
    rules
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Finds or creates the project for an SMTP user that no rule knows yet, so mail sent
/// with credentials gets its own project without any setup (ADR-0012).
pub fn project_for_new_auth_user(
    tx: &Transaction<'_>,
    auth_user: &str,
    now: i64,
) -> Result<Option<ProjectId>, DbError> {
    let (Ok(rule), Ok(name)) = (
        Rule::parse(RuleKind::AuthUser, auth_user),
        ProjectName::parse(auth_user),
    ) else {
        return Ok(None);
    };
    if let Some(owner) = sqlite::rule_owner(tx, &rule)? {
        return Ok(Some(owner));
    }
    let id = match sqlite::id_by_name(tx, &name)? {
        Some(id) => id,
        None => sqlite::insert(tx, &name, true, now)?,
    };
    let mut rules = sqlite::find(tx, id)?
        .map(|project| project.rules)
        .unwrap_or_default();
    rules.push(rule);
    sqlite::replace_rules(tx, id, &rules)?;
    Ok(Some(id))
}
