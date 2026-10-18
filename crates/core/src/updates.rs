//! Opt-in update notice (ADR-0032): once a day, and right after the user turns it on,
//! asks GitHub for the latest release. Nothing is downloaded or installed; the UI only
//! links to the release page. Off by default, since the request reveals the IP address.

use std::sync::{Arc, PoisonError, RwLock};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::error::RecvError;
use tracing::{info, warn};
use utoipa::ToSchema;

use crate::changes::{Change, ChangeFeed};
use crate::clock::now_millis;
use crate::settings::SettingsService;

const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// What the UI shows.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub current_version: String,
    /// The newest published release, once a check succeeded.
    pub latest_version: Option<String>,
    /// Release page of `latest_version`.
    pub url: Option<String>,
    pub update_available: bool,
    /// Unix milliseconds of the last successful check.
    pub checked_at: Option<i64>,
}

#[derive(Clone)]
pub struct UpdateChecker {
    status: Arc<RwLock<UpdateStatus>>,
}

impl Default for UpdateChecker {
    fn default() -> Self {
        Self {
            status: Arc::new(RwLock::new(UpdateStatus {
                current_version: CURRENT_VERSION.to_owned(),
                ..UpdateStatus::default()
            })),
        }
    }
}

impl UpdateChecker {
    pub fn status(&self) -> UpdateStatus {
        self.status
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn record(&self, release: Release, now: i64) {
        let mut status = self.status.write().unwrap_or_else(PoisonError::into_inner);
        status.update_available = is_newer(&release.version, CURRENT_VERSION);
        status.latest_version = Some(release.version);
        status.url = Some(release.url);
        status.checked_at = Some(now);
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Release {
    version: String,
    url: String,
}

/// The part of GitHub's release answer that matters here.
#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
}

fn parse_release(body: &str) -> Option<Release> {
    let release: GithubRelease = serde_json::from_str(body).ok()?;
    Some(Release {
        version: release.tag_name.trim_start_matches('v').to_owned(),
        url: release.html_url,
    })
}

/// `major.minor.patch`; a pre-release suffix (`-beta.1`) counts as older than the
/// release itself. Unreadable versions are never newer.
fn is_newer(candidate: &str, current: &str) -> bool {
    match (version_key(candidate), version_key(current)) {
        (Some(candidate), Some(current)) => candidate > current,
        _ => false,
    }
}

fn version_key(version: &str) -> Option<(u64, u64, u64, bool)> {
    let (core, pre_release) = match version.split_once('-') {
        Some((core, _)) => (core, true),
        None => (version, false),
    };
    let mut parts = core.split('.').map(str::parse::<u64>);
    let key = (
        parts.next()?.ok()?,
        parts.next()?.ok()?,
        parts.next()?.ok()?,
        !pre_release,
    );
    parts.next().is_none().then_some(key)
}

fn latest_release_url() -> String {
    let repository = REPOSITORY.trim_start_matches("https://github.com/");
    format!("https://api.github.com/repos/{repository}/releases/latest")
}

/// Checks when due and whenever the settings change, for the lifetime of the server.
pub async fn watch(checker: UpdateChecker, settings: SettingsService, changes: ChangeFeed) {
    let client = match reqwest::Client::builder()
        .user_agent(concat!("Mailbeaver/", env!("CARGO_PKG_VERSION")))
        .timeout(REQUEST_TIMEOUT)
        .build()
    {
        Ok(client) => client,
        Err(err) => {
            warn!(error = %err, "update check unavailable");
            return;
        }
    };
    let mut feed = changes.subscribe();
    let mut last_attempt: Option<Instant> = None;
    let mut was_enabled = false;
    loop {
        let enabled = settings
            .get()
            .await
            .map(|settings| settings.update_check)
            .unwrap_or(false);
        // Turning the setting on checks at once; otherwise once per interval, also
        // after a failed attempt, so a missing network never causes a request storm.
        let turned_on = enabled && !was_enabled;
        was_enabled = enabled;
        let due = last_attempt.is_none_or(|at| at.elapsed() >= CHECK_INTERVAL);
        if enabled && (due || turned_on) {
            last_attempt = Some(Instant::now());
            check(&client, &checker).await;
        }
        let timer = tokio::time::sleep(CHECK_INTERVAL);
        tokio::pin!(timer);
        loop {
            tokio::select! {
                () = &mut timer => break,
                change = feed.recv() => match change {
                    Ok(Change::SettingsChanged) | Err(RecvError::Lagged(_)) => break,
                    Ok(_) => {}
                    Err(RecvError::Closed) => return,
                },
            }
        }
    }
}

async fn check(client: &reqwest::Client, checker: &UpdateChecker) {
    let response = match client.get(latest_release_url()).send().await {
        Ok(response) => response,
        Err(err) => {
            warn!(error = %err, "update check failed");
            return;
        }
    };
    if !response.status().is_success() {
        info!(
            status = response.status().as_u16(),
            "no published release found"
        );
        return;
    }
    match response
        .text()
        .await
        .ok()
        .as_deref()
        .and_then(parse_release)
    {
        Some(release) => {
            info!(latest = %release.version, current = CURRENT_VERSION, "checked for updates");
            checker.record(release, now_millis());
        }
        None => warn!("update check: unexpected answer from GitHub"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions() {
        assert!(is_newer("0.2.0", "0.1.9"));
        assert!(is_newer("1.0.0", "0.99.99"));
        assert!(is_newer("1.0.0", "1.0.0-beta.2"));
        assert!(!is_newer("1.0.0-beta.2", "1.0.0"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("garbage", "0.1.0"));
        assert!(!is_newer("1.2", "0.1.0"));
    }

    #[test]
    fn reads_github_releases() {
        let body = r#"{"tag_name":"v0.3.1","html_url":"https://github.com/o/r/releases/tag/v0.3.1","assets":[]}"#;
        assert_eq!(
            parse_release(body),
            Some(Release {
                version: "0.3.1".into(),
                url: "https://github.com/o/r/releases/tag/v0.3.1".into()
            })
        );
        assert_eq!(parse_release("{}"), None);
        assert_eq!(
            latest_release_url(),
            "https://api.github.com/repos/mehmetmuejde/mailbeaver/releases/latest"
        );
    }

    #[test]
    fn records_the_newest_release() {
        let checker = UpdateChecker::default();
        assert_eq!(checker.status().current_version, CURRENT_VERSION);
        checker.record(
            Release {
                version: "99.0.0".into(),
                url: "https://example.test".into(),
            },
            5,
        );
        let status = checker.status();
        assert!(status.update_available);
        assert_eq!(status.checked_at, Some(5));
    }
}
