//! Captured mail: storage, folders, search, the trash and its retention.

mod domain;
mod http;
mod links;
mod mime;
mod retention;
mod search;
mod service;
mod sqlite;

pub use domain::{
    AuthUser, Counts, DeleteOutcome, Envelope, Folder, ListQuery, MessageDetail, MessageId,
    MessageSummary, ProjectRef,
};
pub(crate) use http::router;
pub use links::extract_links;
pub use retention::{TRASH_CHECK_INTERVAL, auto_empty_trash};
pub use search::SearchQuery;
pub use service::MessageService;
pub(crate) use sqlite::{counts_by_project, reassign_projects};
