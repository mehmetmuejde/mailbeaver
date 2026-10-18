//! Starts the SMTP and HTTP servers and the trash retention. Independent of Tauri, so a
//! headless host can reuse it (ADR-0001).

use std::net::SocketAddr;

use anyhow::Context;
use axum::Router;
use axum::http::HeaderValue;
use tokio::net::TcpListener;
use tracing::{error, info, warn};

use crate::capture::Capture;
use crate::changes::ChangeFeed;
use crate::config::Config;
use crate::db::Database;
use crate::http::{self, ApiConfig, ServerInfo, Services};
use crate::links::LinkChecker;
use crate::logs::{self, LogReceiver, LogService};
use crate::maintenance::MaintenanceService;
use crate::messages::{self, MessageService};
use crate::projects::ProjectService;
use crate::settings::SettingsService;
use crate::smtp::{self, SmtpConfig};
use crate::stats::Stats;
use crate::updates::{self, UpdateChecker};

const MAX_SMTP_CONNECTIONS: usize = 64;
const SMTP_HOSTNAME: &str = "mailbeaver";

/// Origins the desktop UI runs on: Tauri on macOS/Linux, Tauri on Windows, Vite dev server.
const UI_ORIGINS: [&str; 3] = [
    "tauri://localhost",
    "http://tauri.localhost",
    "http://localhost:1420",
];

/// Addresses the servers actually listen on, plus the services they share.
#[derive(Clone)]
pub struct Running {
    pub smtp: SocketAddr,
    pub http: SocketAddr,
    pub services: Services,
}

/// Which program runs the core; the UI offers what that host can do, e.g. a restart.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HostKind {
    Desktop,
    #[default]
    Server,
}

impl HostKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Server => "server",
        }
    }
}

/// What a host adds to the core's servers.
#[derive(Default)]
pub struct Host {
    pub kind: HostKind,
    /// Answers every path outside the API, e.g. the built UI of the headless server.
    pub ui: Option<Router>,
    /// Log events of the host's `tracing` subscriber, stored for Settings → Logs.
    pub logs: Option<LogReceiver>,
}

/// Opens the database, binds both ports and spawns the servers. Fails early when a
/// port is taken, so the caller can tell the user.
pub async fn start(config: Config, host: Host) -> anyhow::Result<Running> {
    let path = config.database.clone();
    let db = tokio::task::spawn_blocking(move || Database::open(&path))
        .await?
        .with_context(|| format!("cannot open database {}", config.database.display()))?;
    let changes = ChangeFeed::new();
    let settings = SettingsService::new(db.clone(), changes.clone());
    // Ports and limits saved in the app, unless an environment variable decided.
    let config = config.with_saved(&settings.server().await?);
    let stats = Stats::new();
    let services = Services {
        messages: MessageService::new(
            db.clone(),
            changes.clone(),
            stats.clone(),
            config.max_messages,
        ),
        projects: ProjectService::new(db.clone(), changes.clone()),
        settings,
        maintenance: MaintenanceService::new(db.clone(), changes.clone()),
        stats,
        links: LinkChecker::new().context("cannot create the link checker")?,
        logs: LogService::new(db.clone()),
        capture: Capture::new(changes.clone()),
        updates: UpdateChecker::default(),
        db,
        changes,
    };

    let smtp_listener = TcpListener::bind((config.bind, config.smtp_port))
        .await
        .with_context(|| format!("SMTP port {} is not available", config.smtp_port))?;
    let http_listener = TcpListener::bind((config.bind, config.http_port))
        .await
        .with_context(|| format!("HTTP port {} is not available", config.http_port))?;
    let running = Running {
        smtp: smtp_listener.local_addr()?,
        http: http_listener.local_addr()?,
        services: services.clone(),
    };

    let router = http::router(
        services.clone(),
        ApiConfig {
            allowed_origins: UI_ORIGINS
                .into_iter()
                .map(HeaderValue::from_static)
                .collect(),
            allowed_hosts: allowed_hosts(&config, running.http.port()),
            ui: host.ui,
            info: ServerInfo {
                version: env!("CARGO_PKG_VERSION").to_owned(),
                smtp_address: config
                    .public_smtp_address
                    .clone()
                    .unwrap_or_else(|| running.smtp.to_string()),
                http_address: config
                    .public_http_address
                    .clone()
                    .unwrap_or_else(|| running.http.to_string()),
                database: config.database.display().to_string(),
                max_messages: config.max_messages,
                max_message_bytes: config.max_message_bytes,
                from_environment: config.from_env.names(),
                host: host.kind.as_str().to_owned(),
            },
        },
    );

    let smtp_config = SmtpConfig {
        hostname: SMTP_HOSTNAME.to_owned(),
        max_message_bytes: config.max_message_bytes,
        max_connections: MAX_SMTP_CONNECTIONS,
        capture: services.capture.clone(),
    };
    tokio::spawn(messages::auto_empty_trash(
        services.messages.clone(),
        services.settings.clone(),
        services.changes.clone(),
        messages::TRASH_CHECK_INTERVAL,
    ));
    tokio::spawn(updates::watch(
        services.updates.clone(),
        services.settings.clone(),
        services.changes.clone(),
    ));
    if let Some(receiver) = host.logs {
        tokio::spawn(logs::persist(receiver, services.logs.clone()));
    }
    tokio::spawn(smtp::serve(smtp_listener, services.messages, smtp_config));
    tokio::spawn(async move {
        if let Err(err) = axum::serve(http_listener, router).await {
            error!(error = %err, "http server stopped");
        }
    });

    info!(smtp = %running.smtp, http = %running.http, "mailbeaver listening");
    Ok(running)
}

/// The `Host` check: loopback names always, plus `MAILBEAVER_ALLOWED_HOSTS`. Bound to
/// another address without that list, the check is off — the host name is unknown.
fn allowed_hosts(config: &Config, port: u16) -> Option<Vec<String>> {
    if !config.bind.is_loopback() && config.allowed_hosts.is_empty() {
        warn!(
            bind = %config.bind,
            "Host check is off; set MAILBEAVER_ALLOWED_HOSTS to the names the UI is opened by"
        );
        return None;
    }
    let mut hosts = loopback_hosts(port);
    hosts.extend(config.allowed_hosts.iter().cloned());
    Some(hosts)
}

fn loopback_hosts(port: u16) -> Vec<String> {
    ["127.0.0.1", "localhost", "[::1]"]
        .into_iter()
        .map(|host| format!("{host}:{port}"))
        .collect()
}
