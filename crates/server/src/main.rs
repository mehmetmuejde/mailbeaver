//! Mailbeaver without a desktop (ADR-0028): SMTP server, HTTP API, MCP server and the UI
//! on one HTTP port, configured by `MAILBEAVER_*` environment variables. Made for Docker.
//!
//! `mailbeaver-server healthcheck` asks the running server for `/api/health` and exits
//! with 0 or 1; the container image has neither a shell nor curl for that.

mod ui;

use std::env;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use mailbeaver_core::config::Config;
use mailbeaver_core::logs::{self, LogReceiver};
use mailbeaver_core::server::{self, Host, HostKind};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Relative to the working directory; the container image sets `/data`.
const DEFAULT_DATA_DIR: &str = "data";
const HEALTHCHECK_TIMEOUT: Duration = Duration::from_secs(3);

#[tokio::main]
async fn main() -> ExitCode {
    // Console output plus the database for Settings → Logs.
    let (database_layer, receiver) = logs::database_layer();
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .with(database_layer)
        .init();

    if env::args().nth(1).as_deref() == Some("healthcheck") {
        return healthcheck();
    }
    match run(receiver).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!(error = format!("{err:#}"), "mailbeaver stopped");
            ExitCode::FAILURE
        }
    }
}

async fn run(logs: LogReceiver) -> anyhow::Result<()> {
    let data_dir = env::var_os("MAILBEAVER_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DATA_DIR));
    std::fs::create_dir_all(&data_dir)?;
    let config = Config::from_env(data_dir)?;
    let host = Host {
        kind: HostKind::Server,
        ui: Some(ui::router()),
        logs: Some(logs),
    };
    let running = server::start(config, host).await?;
    info!(url = format!("http://{}", running.http), "UI ready");
    shutdown_signal().await;
    info!("shutting down");
    Ok(())
}

/// Ctrl+C, or SIGTERM from `docker stop`.
async fn shutdown_signal() {
    let interrupt = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            error!(error = %err, "cannot listen for Ctrl+C");
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(err) => {
                error!(error = %err, "cannot listen for SIGTERM");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = interrupt => {}
        () = terminate => {}
    }
}

fn healthcheck() -> ExitCode {
    match check_health() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!(error = format!("{err:#}"), "health check failed");
            ExitCode::FAILURE
        }
    }
}

fn check_health() -> anyhow::Result<()> {
    let config = Config::from_env(PathBuf::new())?;
    // A server bound to every address is reachable on loopback.
    let ip = if config.bind.is_unspecified() {
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    } else {
        config.bind
    };
    let address = SocketAddr::new(ip, config.http_port);
    let mut stream = TcpStream::connect_timeout(&address, HEALTHCHECK_TIMEOUT)?;
    stream.set_read_timeout(Some(HEALTHCHECK_TIMEOUT))?;
    write!(
        stream,
        "GET /api/health HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n"
    )?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    anyhow::ensure!(
        response.starts_with("HTTP/1.1 200"),
        "unexpected answer: {}",
        response.lines().next().unwrap_or_default()
    );
    Ok(())
}
