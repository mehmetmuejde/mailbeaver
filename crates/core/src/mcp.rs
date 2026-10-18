//! MCP server for agents such as Claude Code: read the caught mail, wait for a new one,
//! open its links. Served over streamable HTTP at `/mcp`; no tool deletes anything.

use std::sync::Arc;
use std::time::Duration;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_handler, tool_router};
use serde::Deserialize;
use tokio::sync::broadcast::error::RecvError;

use crate::changes::{Change, ChangeFeed};
use crate::messages::{self, Folder, ListQuery, MessageId, MessageService, SearchQuery};

const DEFAULT_LIMIT: u32 = 20;
const MAX_LIMIT: u32 = 50;
const DEFAULT_WAIT_SECONDS: u64 = 30;
const MAX_WAIT_SECONDS: u64 = 60;

/// The MCP service to mount at `/mcp`.
pub fn service(
    messages: MessageService,
    changes: ChangeFeed,
) -> StreamableHttpService<MailbeaverMcp, LocalSessionManager> {
    StreamableHttpService::new(
        move || Ok(MailbeaverMcp::new(messages.clone(), changes.clone())),
        Arc::new(LocalSessionManager::default()),
        // Browsers always send an Origin; agents do not. Rejecting every Origin keeps
        // web pages away from the tools.
        StreamableHttpServerConfig::default().enforce_origin_validation(),
    )
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct ListArgs {
    /// `inbox`, `archive` or `trash`; all folders when left out.
    folder: Option<String>,
    /// Full-text search over subject, sender, recipients and body; every word must
    /// match. Filters: `from:`, `to:`, `subject:`, `body:`, `is:unread`, `is:read`,
    /// `has:attachment`, `project:<name>` or `project:none`, `after:YYYY-MM-DD`,
    /// `before:YYYY-MM-DD` (UTC days). Quote values with spaces: `subject:"order 12"`.
    query: Option<String>,
    /// How many messages, newest first (default 20, at most 50).
    limit: Option<u32>,
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct MessageArgs {
    /// The message id from `list_messages` or `wait_for_message`.
    id: i64,
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct WaitArgs {
    /// Only a message with a recipient containing this text, e.g. `user@example.com`.
    to: Option<String>,
    /// Only a message whose subject contains this text.
    subject: Option<String>,
    /// How long to wait (default 30, at most 60 seconds).
    timeout_seconds: Option<u64>,
}

#[derive(Clone)]
pub struct MailbeaverMcp {
    messages: MessageService,
    changes: ChangeFeed,
}

fn json(value: &impl serde::Serialize) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![ContentBlock::json(value)?]))
}

fn internal(err: impl std::fmt::Display) -> McpError {
    McpError::internal_error(err.to_string(), None)
}

fn contains(haystack: &str, needle: Option<&str>) -> bool {
    needle.is_none_or(|needle| haystack.to_lowercase().contains(&needle.to_lowercase()))
}

#[tool_router]
impl MailbeaverMcp {
    fn new(messages: MessageService, changes: ChangeFeed) -> Self {
        Self { messages, changes }
    }

    #[tool(
        description = "List caught messages, newest first, optionally in one folder or matching a full-text query."
    )]
    async fn list_messages(
        &self,
        Parameters(args): Parameters<ListArgs>,
    ) -> Result<CallToolResult, McpError> {
        let folder = args
            .folder
            .as_deref()
            .map(str::parse::<Folder>)
            .transpose()
            .map_err(|err| McpError::invalid_params(err.to_string(), None))?;
        let page = self
            .messages
            .list(ListQuery {
                folder,
                search: args.query.as_deref().and_then(SearchQuery::parse),
                limit: args.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT),
                ..ListQuery::default()
            })
            .await
            .map_err(internal)?;
        json(&page)
    }

    #[tool(
        description = "Read one message: headers, plain text, HTML, recipients and attachment list."
    )]
    async fn get_message(
        &self,
        Parameters(args): Parameters<MessageArgs>,
    ) -> Result<CallToolResult, McpError> {
        let detail = self
            .messages
            .detail(MessageId(args.id))
            .await
            .map_err(internal)?
            .ok_or_else(|| McpError::resource_not_found("no such message", None))?;
        json(&detail)
    }

    #[tool(
        description = "The links of a message (http, https, mailto), e.g. to confirm a sign-up or reset a password."
    )]
    async fn get_message_links(
        &self,
        Parameters(args): Parameters<MessageArgs>,
    ) -> Result<CallToolResult, McpError> {
        let detail = self
            .messages
            .detail(MessageId(args.id))
            .await
            .map_err(internal)?
            .ok_or_else(|| McpError::resource_not_found("no such message", None))?;
        json(&messages::extract_links(
            detail.body.html.as_deref(),
            detail.body.text.as_deref(),
        ))
    }

    #[tool(
        description = "Wait until a new message arrives (optionally for a recipient or subject) and return it. Use it right after the app under test sends mail."
    )]
    async fn wait_for_message(
        &self,
        Parameters(args): Parameters<WaitArgs>,
    ) -> Result<CallToolResult, McpError> {
        let seconds = args
            .timeout_seconds
            .unwrap_or(DEFAULT_WAIT_SECONDS)
            .clamp(1, MAX_WAIT_SECONDS);
        let mut feed = self.changes.subscribe();
        let wait = async {
            loop {
                match feed.recv().await {
                    Ok(Change::Received { id }) => {
                        let Some(detail) = self.messages.detail(id).await.map_err(internal)? else {
                            continue;
                        };
                        let recipients =
                            detail.summary.to.join(" ") + " " + &detail.envelope_to.join(" ");
                        if contains(&recipients, args.to.as_deref())
                            && contains(&detail.summary.subject, args.subject.as_deref())
                        {
                            return Ok(Some(detail));
                        }
                    }
                    Ok(_) | Err(RecvError::Lagged(_)) => {}
                    Err(RecvError::Closed) => return Ok(None),
                }
            }
        };
        match tokio::time::timeout(Duration::from_secs(seconds), wait).await {
            Ok(Ok(Some(detail))) => json(&detail),
            Ok(Err(err)) => Err(err),
            Ok(Ok(None)) | Err(_) => Ok(CallToolResult::success(vec![ContentBlock::text(
                format!("No matching message arrived within {seconds} seconds."),
            )])),
        }
    }

    #[tool(description = "Mark a message as read.")]
    async fn mark_message_read(
        &self,
        Parameters(args): Parameters<MessageArgs>,
    ) -> Result<CallToolResult, McpError> {
        if self
            .messages
            .mark_read(MessageId(args.id))
            .await
            .map_err(internal)?
        {
            Ok(CallToolResult::success(vec![ContentBlock::text(
                "Marked as read.",
            )]))
        } else {
            Err(McpError::resource_not_found("no such message", None))
        }
    }
}

#[tool_handler]
impl ServerHandler for MailbeaverMcp {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("mailbeaver", env!("CARGO_PKG_VERSION")))
            .with_instructions(
                "Mailbeaver catches the mail an application sends during development. Use \
                 wait_for_message after triggering an email, then get_message or \
                 get_message_links to read it.",
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::messages::Envelope;
    use crate::stats::Stats;

    fn server() -> MailbeaverMcp {
        let changes = ChangeFeed::new();
        let messages = MessageService::new(
            Database::open_in_memory().unwrap(),
            changes.clone(),
            Stats::new(),
            10,
        );
        MailbeaverMcp::new(messages, changes)
    }

    async fn receive(server: &MailbeaverMcp, to: &str, subject: &str) -> MessageId {
        server
            .messages
            .receive(
                Envelope {
                    rcpt_to: vec![to.into()],
                    ..Envelope::default()
                },
                format!(
                    "To: {to}\r\nSubject: {subject}\r\nContent-Type: text/html\r\n\r\n\
                     <a href=\"https://app.example/confirm?t=1\">Confirm</a>\r\n"
                )
                .into_bytes(),
            )
            .await
            .unwrap()
    }

    fn text(result: &CallToolResult) -> String {
        serde_json::to_string(&result.content).unwrap()
    }

    #[tokio::test]
    async fn lists_reads_and_extracts_links() {
        let server = server();
        let id = receive(&server, "dev@app.example", "Welcome").await;

        let list = server
            .list_messages(Parameters(ListArgs {
                folder: Some("inbox".into()),
                query: Some("welcome".into()),
                limit: None,
            }))
            .await
            .unwrap();
        assert!(text(&list).contains("Welcome"));

        let links = server
            .get_message_links(Parameters(MessageArgs { id: id.0 }))
            .await
            .unwrap();
        assert!(text(&links).contains("https://app.example/confirm?t=1"));

        assert!(
            server
                .get_message(Parameters(MessageArgs { id: 99 }))
                .await
                .is_err()
        );
        assert!(
            server
                .list_messages(Parameters(ListArgs {
                    folder: Some("spam".into()),
                    query: None,
                    limit: None,
                }))
                .await
                .is_err()
        );
        server
            .mark_message_read(Parameters(MessageArgs { id: id.0 }))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn waits_for_a_matching_message() {
        let server = server();
        let sender = server.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            receive(&sender, "other@app.example", "Ignore me").await;
            receive(&sender, "dev@app.example", "Your code").await;
        });

        let result = server
            .wait_for_message(Parameters(WaitArgs {
                to: Some("DEV@app.example".into()),
                subject: None,
                timeout_seconds: Some(5),
            }))
            .await
            .unwrap();
        assert!(text(&result).contains("Your code"));

        let timed_out = server
            .wait_for_message(Parameters(WaitArgs {
                to: None,
                subject: Some("never".into()),
                timeout_seconds: Some(1),
            }))
            .await
            .unwrap();
        assert!(text(&timed_out).contains("No matching message"));
    }
}
