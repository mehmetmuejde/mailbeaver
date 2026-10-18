//! Typed configuration: `MAILBEAVER_*` environment variables first, then the ports and
//! limits saved in the app, then defaults (ADR-0031).

use std::env;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;

use anyhow::Context;

use crate::settings::ServerSettings;

const DEFAULT_SMTP_PORT: u16 = 1026;
const DEFAULT_HTTP_PORT: u16 = 8026;
const DEFAULT_MAX_MESSAGE_MB: usize = 25;
const DEFAULT_MAX_MESSAGES: u32 = 5000;
const DATABASE_FILE: &str = "mailbeaver.db";

#[derive(Debug, Clone)]
pub struct Config {
    pub bind: IpAddr,
    pub smtp_port: u16,
    pub http_port: u16,
    pub max_message_bytes: usize,
    pub max_messages: u32,
    pub database: PathBuf,
    /// Extra `Host` header values the HTTP API accepts, e.g. the name a container is
    /// reached by. Loopback names are always accepted.
    pub allowed_hosts: Vec<String>,
    /// Addresses shown in the UI instead of the bound ones, e.g. behind port mapping.
    pub public_smtp_address: Option<String>,
    pub public_http_address: Option<String>,
    /// Which saved settings an environment variable overrides.
    pub from_env: EnvOverrides,
}

/// Values set by an environment variable; saved settings cannot change them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EnvOverrides {
    pub smtp_port: bool,
    pub http_port: bool,
    pub max_messages: bool,
    pub max_message_mb: bool,
}

impl EnvOverrides {
    /// The names of the overridden settings as the API spells them.
    pub fn names(self) -> Vec<String> {
        [
            ("smtpPort", self.smtp_port),
            ("httpPort", self.http_port),
            ("maxMessages", self.max_messages),
            ("maxMessageMb", self.max_message_mb),
        ]
        .into_iter()
        .filter(|(_, set)| *set)
        .map(|(name, _)| name.to_owned())
        .collect()
    }
}

impl Config {
    /// Reads `MAILBEAVER_*` variables, falling back to defaults; the database lives in
    /// `data_dir`.
    pub fn from_env(data_dir: PathBuf) -> anyhow::Result<Self> {
        let smtp_port: Option<u16> = env_opt("MAILBEAVER_SMTP_PORT")?;
        let http_port: Option<u16> = env_opt("MAILBEAVER_HTTP_PORT")?;
        let max_messages: Option<u32> = env_opt("MAILBEAVER_MAX_MESSAGES")?;
        let max_message_mb: Option<usize> = env_opt("MAILBEAVER_MAX_MESSAGE_MB")?;
        Ok(Self {
            bind: env_or("MAILBEAVER_BIND", IpAddr::V4(Ipv4Addr::LOCALHOST))?,
            smtp_port: smtp_port.unwrap_or(DEFAULT_SMTP_PORT),
            http_port: http_port.unwrap_or(DEFAULT_HTTP_PORT),
            max_message_bytes: megabytes(max_message_mb.unwrap_or(DEFAULT_MAX_MESSAGE_MB))
                .context("MAILBEAVER_MAX_MESSAGE_MB is too large")?,
            max_messages: max_messages.unwrap_or(DEFAULT_MAX_MESSAGES),
            from_env: EnvOverrides {
                smtp_port: smtp_port.is_some(),
                http_port: http_port.is_some(),
                max_messages: max_messages.is_some(),
                max_message_mb: max_message_mb.is_some(),
            },
            database: data_dir.join(DATABASE_FILE),
            allowed_hosts: env::var("MAILBEAVER_ALLOWED_HOSTS")
                .map(|hosts| list(&hosts))
                .unwrap_or_default(),
            public_smtp_address: non_empty_env("MAILBEAVER_PUBLIC_SMTP_ADDRESS"),
            public_http_address: non_empty_env("MAILBEAVER_PUBLIC_HTTP_ADDRESS"),
        })
    }
}

impl Config {
    /// Applies what was saved in the app wherever no environment variable decided.
    pub fn with_saved(mut self, saved: &ServerSettings) -> Self {
        if !self.from_env.smtp_port
            && let Some(port) = saved.smtp_port
        {
            self.smtp_port = port;
        }
        if !self.from_env.http_port
            && let Some(port) = saved.http_port
        {
            self.http_port = port;
        }
        if !self.from_env.max_messages
            && let Some(max) = saved.max_messages
        {
            self.max_messages = max;
        }
        if !self.from_env.max_message_mb
            && let Some(bytes) = saved
                .max_message_mb
                .and_then(|mb| megabytes(usize::try_from(mb).ok()?))
        {
            self.max_message_bytes = bytes;
        }
        self
    }
}

fn megabytes(mb: usize) -> Option<usize> {
    mb.checked_mul(1024 * 1024)
}

/// Comma-separated values, trimmed, empty ones dropped.
fn list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect()
}

fn non_empty_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn env_opt<T>(name: &str) -> anyhow::Result<Option<T>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match env::var(name) {
        Ok(value) => value
            .trim()
            .parse()
            .map(Some)
            .with_context(|| format!("invalid value for {name}")),
        Err(_) => Ok(None),
    }
}

fn env_or<T>(name: &str, default: T) -> anyhow::Result<T>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match env::var(name) {
        Ok(value) => value
            .trim()
            .parse()
            .with_context(|| format!("invalid value for {name}")),
        Err(_) => Ok(default),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults() -> Config {
        Config {
            bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
            smtp_port: DEFAULT_SMTP_PORT,
            http_port: DEFAULT_HTTP_PORT,
            max_message_bytes: 25 * 1024 * 1024,
            max_messages: DEFAULT_MAX_MESSAGES,
            database: PathBuf::new(),
            allowed_hosts: Vec::new(),
            public_smtp_address: None,
            public_http_address: None,
            from_env: EnvOverrides::default(),
        }
    }

    #[test]
    fn saved_settings_apply_unless_the_environment_decided() {
        let saved = ServerSettings {
            smtp_port: Some(2525),
            http_port: Some(8080),
            max_messages: Some(100),
            max_message_mb: Some(5),
        };
        let config = defaults().with_saved(&saved);
        assert_eq!((config.smtp_port, config.http_port), (2525, 8080));
        assert_eq!(config.max_messages, 100);
        assert_eq!(config.max_message_bytes, 5 * 1024 * 1024);

        let pinned = Config {
            from_env: EnvOverrides {
                smtp_port: true,
                max_message_mb: true,
                ..EnvOverrides::default()
            },
            ..defaults()
        }
        .with_saved(&saved);
        assert_eq!(
            (pinned.smtp_port, pinned.http_port),
            (DEFAULT_SMTP_PORT, 8080)
        );
        assert_eq!(pinned.max_message_bytes, 25 * 1024 * 1024);
        assert_eq!(pinned.from_env.names(), ["smtpPort", "maxMessageMb"]);

        assert_eq!(
            defaults().with_saved(&ServerSettings::default()).smtp_port,
            DEFAULT_SMTP_PORT
        );
    }

    #[test]
    fn splits_lists() {
        assert_eq!(
            list(" mail.test:8026, ,localhost:8026 "),
            ["mail.test:8026", "localhost:8026"]
        );
        assert!(list("").is_empty());
    }
}
