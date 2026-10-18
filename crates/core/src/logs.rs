//! Log of the backend and the UI, shown in Settings → Logs (ADR-0030). Backend events
//! arrive through a `tracing` layer, UI entries through the API.

mod domain;
mod http;
mod layer;
mod service;
mod sqlite;

pub(crate) use http::router;
pub(crate) use layer::persist;
pub use layer::{DatabaseLayer, LogReceiver, database_layer};
pub use service::LogService;
