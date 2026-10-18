//! Mailbeaver catches every mail an application sends during development and shows it in
//! a mail client instead of delivering it. It never relays mail.
//!
//! The core: SMTP server, storage, HTTP API and MCP server. It does not depend on Tauri;
//! the desktop app (`src-tauri`) and the headless server (`crates/server`) are its hosts
//! (ADR-0001, ADR-0028).

pub mod capture;
pub mod changes;
pub mod clock;
pub mod config;
mod db;
pub mod http;
mod links;
pub mod logs;
mod mailpit;
mod maintenance;
mod mcp;
pub mod messages;
pub mod notifications;
pub mod projects;
pub mod server;
pub mod settings;
mod smtp;
mod stats;
pub mod updates;
