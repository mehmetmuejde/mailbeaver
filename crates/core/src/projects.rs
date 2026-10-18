//! Projects group captured mail. Rules (sender address, sender domain, SMTP user)
//! assign every incoming message to at most one project (ADR-0012).

mod domain;
mod http;
mod service;
mod sqlite;

pub use domain::{ProjectId, ProjectRules};
#[cfg(test)]
pub use domain::{Rule, RuleKind};
pub(crate) use http::router;
pub use service::ProjectService;
pub(crate) use service::project_for_new_auth_user;
pub(crate) use sqlite::load_rules;
