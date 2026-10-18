//! Checks the links of a message with HTTP requests, only when the user asks for it.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Method, StatusCode, Url, redirect};
use serde::Serialize;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use utoipa::ToSchema;

const TIMEOUT: Duration = Duration::from_secs(5);
const MAX_REDIRECTS: usize = 5;
const MAX_PARALLEL: usize = 8;
pub const MAX_LINKS: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LinkStatus {
    pub url: String,
    /// HTTP status of the final response; `None` when no response arrived.
    pub status: Option<u16>,
    /// A 2xx or 3xx answer.
    pub ok: bool,
    /// Why no response arrived, e.g. a timeout.
    pub error: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum LinkCheckError {
    #[error("at most {MAX_LINKS} links can be checked at once")]
    TooMany,
    #[error("{0:?} is not an http or https URL")]
    InvalidUrl(String),
    #[error("the HTTP client could not be created")]
    Client(#[from] reqwest::Error),
}

#[derive(Clone)]
pub struct LinkChecker {
    client: reqwest::Client,
}

impl LinkChecker {
    /// No cookies are kept: every check is anonymous.
    pub fn new() -> Result<Self, LinkCheckError> {
        let client = reqwest::Client::builder()
            .timeout(TIMEOUT)
            .redirect(redirect::Policy::limited(MAX_REDIRECTS))
            .user_agent(concat!("Mailbeaver link check/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self { client })
    }

    /// Checks every URL, at most a few at a time; the results keep the input order.
    pub async fn check_all(&self, urls: Vec<String>) -> Result<Vec<LinkStatus>, LinkCheckError> {
        if urls.len() > MAX_LINKS {
            return Err(LinkCheckError::TooMany);
        }
        let parsed = urls
            .iter()
            .map(|raw| match Url::parse(raw) {
                Ok(url) if matches!(url.scheme(), "http" | "https") => Ok(url),
                _ => Err(LinkCheckError::InvalidUrl(raw.clone())),
            })
            .collect::<Result<Vec<_>, _>>()?;

        let permits = Arc::new(Semaphore::new(MAX_PARALLEL));
        let mut checks = JoinSet::new();
        for (index, url) in parsed.into_iter().enumerate() {
            let client = self.client.clone();
            let permits = Arc::clone(&permits);
            checks.spawn(async move {
                // The semaphore is never closed, so a permit always comes.
                let _permit = permits.acquire_owned().await;
                (index, check(&client, url).await)
            });
        }
        let mut results: Vec<Option<LinkStatus>> = vec![None; urls.len()];
        while let Some(joined) = checks.join_next().await {
            if let Ok((index, status)) = joined
                && let Some(slot) = results.get_mut(index)
            {
                *slot = Some(status);
            }
        }
        Ok(results
            .into_iter()
            .zip(urls)
            .map(|(result, url)| {
                result.unwrap_or(LinkStatus {
                    url,
                    status: None,
                    ok: false,
                    error: Some("check failed".to_owned()),
                })
            })
            .collect())
    }
}

/// HEAD first; servers that do not support it get a GET.
async fn check(client: &reqwest::Client, url: Url) -> LinkStatus {
    let mut response = client.request(Method::HEAD, url.clone()).send().await;
    if let Ok(head) = &response
        && matches!(
            head.status(),
            StatusCode::METHOD_NOT_ALLOWED | StatusCode::NOT_IMPLEMENTED
        )
    {
        response = client.get(url.clone()).send().await;
    }
    match response {
        Ok(response) => {
            let status = response.status();
            LinkStatus {
                url: url.to_string(),
                status: Some(status.as_u16()),
                ok: status.is_success() || status.is_redirection(),
                error: None,
            }
        }
        Err(err) => LinkStatus {
            url: url.to_string(),
            status: None,
            ok: false,
            error: Some(describe(&err).to_owned()),
        },
    }
}

fn describe(err: &reqwest::Error) -> &'static str {
    if err.is_timeout() {
        "timed out"
    } else if err.is_connect() {
        "could not connect"
    } else if err.is_redirect() {
        "too many redirects"
    } else {
        "request failed"
    }
}

#[cfg(test)]
mod tests {
    use axum::Router;
    use axum::http::StatusCode;
    use axum::routing::{get, head};
    use tokio::net::TcpListener;

    use super::*;

    async fn serve() -> String {
        let router = Router::new()
            .route("/ok", get(|| async { "fine" }))
            .route("/missing", get(|| async { StatusCode::NOT_FOUND }))
            .route(
                "/no-head",
                head(|| async { StatusCode::METHOD_NOT_ALLOWED }).get(|| async { "fine" }),
            )
            .route(
                "/moved",
                get(|| async { axum::response::Redirect::temporary("/ok") }),
            );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, router).await });
        format!("http://{addr}")
    }

    #[tokio::test]
    async fn reports_status_per_link_in_order() {
        let base = serve().await;
        let checker = LinkChecker::new().unwrap();
        let urls = ["/ok", "/missing", "/no-head", "/moved"]
            .iter()
            .map(|path| format!("{base}{path}"))
            .collect();

        let results = checker.check_all(urls).await.unwrap();

        let summary: Vec<_> = results.iter().map(|r| (r.status, r.ok)).collect();
        assert_eq!(
            summary,
            [
                (Some(200), true),
                (Some(404), false),
                (Some(200), true),
                (Some(200), true)
            ]
        );
        assert!(results[0].url.ends_with("/ok"));
    }

    #[tokio::test]
    async fn reports_unreachable_hosts_and_rejects_bad_input() {
        let checker = LinkChecker::new().unwrap();
        // Port 9 (discard) is closed on loopback.
        let results = checker
            .check_all(vec!["http://127.0.0.1:9/".into()])
            .await
            .unwrap();
        assert_eq!(results[0].status, None);
        assert_eq!(results[0].error.as_deref(), Some("could not connect"));

        assert!(matches!(
            checker.check_all(vec!["mailto:a@b.example".into()]).await,
            Err(LinkCheckError::InvalidUrl(_))
        ));
        assert!(matches!(
            checker
                .check_all(vec!["http://x.example/".into(); MAX_LINKS + 1])
                .await,
            Err(LinkCheckError::TooMany)
        ));
    }
}
