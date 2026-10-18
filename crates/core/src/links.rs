//! Link check: requests the links found in a message and reports their HTTP status.
//! Runs only on demand; no cookies, short timeouts.

mod checker;
mod http;

pub use checker::LinkChecker;
pub(crate) use http::router;
