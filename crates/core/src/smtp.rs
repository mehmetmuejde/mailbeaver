//! Minimal SMTP server (RFC 5321 subset) that accepts every message and hands it to
//! the message service. It never relays. Any AUTH credentials are accepted and the
//! username can select a project; passwords are never kept. TLS is not offered.

use std::io;
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use tokio::io::{
    AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader,
};
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

use crate::capture::Capture;
use crate::messages::{AuthUser, Envelope, MessageService};

const IDLE_TIMEOUT: Duration = Duration::from_secs(300);
const MAX_COMMAND_LINE: usize = 4096;
const MAX_RECIPIENTS: usize = 100;
const ACCEPT_ERROR_BACKOFF: Duration = Duration::from_millis(100);

#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub hostname: String,
    pub max_message_bytes: usize,
    pub max_connections: usize,
    /// While paused, `MAIL` is answered with a temporary error.
    pub capture: Capture,
}

/// Accepts connections until the listener fails permanently.
pub async fn serve(listener: TcpListener, messages: MessageService, config: SmtpConfig) {
    let config = Arc::new(config);
    let permits = Arc::new(Semaphore::new(config.max_connections));
    loop {
        let Ok(permit) = Arc::clone(&permits).acquire_owned().await else {
            return;
        };
        match listener.accept().await {
            Ok((stream, peer)) => {
                let messages = messages.clone();
                let config = Arc::clone(&config);
                tokio::spawn(async move {
                    debug!(%peer, "smtp connection opened");
                    if let Err(err) = handle(stream, &messages, &config).await {
                        debug!(%peer, error = %err, "smtp connection ended with error");
                    }
                    drop(permit);
                });
            }
            Err(err) => {
                // Typically EMFILE; back off instead of spinning.
                warn!(error = %err, "smtp accept failed");
                tokio::time::sleep(ACCEPT_ERROR_BACKOFF).await;
            }
        }
    }
}

/// Runs one SMTP session on any byte stream.
pub async fn handle<S>(stream: S, messages: &MessageService, config: &SmtpConfig) -> io::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let (reader, writer) = tokio::io::split(stream);
    let mut session = Session {
        reader: BufReader::new(reader),
        writer,
        messages,
        config,
        sender: None,
        recipients: Vec::new(),
        auth_user: None,
        line: Vec::new(),
    };
    session.run().await
}

enum Line {
    Complete,
    TooLong,
    Closed,
}

struct Session<'a, R, W> {
    reader: BufReader<R>,
    writer: W,
    messages: &'a MessageService,
    config: &'a SmtpConfig,
    sender: Option<String>,
    recipients: Vec<String>,
    /// Set by AUTH for the rest of the session; `RSET` and `EHLO` keep it.
    auth_user: Option<AuthUser>,
    line: Vec<u8>,
}

impl<R, W> Session<'_, R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    async fn run(&mut self) -> io::Result<()> {
        let greeting = format!("220 {} ESMTP mailbeaver ready", self.config.hostname);
        self.reply(&greeting).await?;
        loop {
            match self.next_line(MAX_COMMAND_LINE).await? {
                Line::Complete => {}
                Line::TooLong => return self.reply("500 5.5.6 Line too long").await,
                Line::Closed => return Ok(()),
            }
            let command = String::from_utf8_lossy(&self.line).trim_end().to_owned();
            let (verb, arg) = command.split_once(' ').unwrap_or((command.as_str(), ""));
            match verb.to_ascii_uppercase().as_str() {
                "EHLO" => self.ehlo(arg).await?,
                "HELO" => {
                    self.reset();
                    let reply = format!("250 {}", self.config.hostname);
                    self.reply(&reply).await?;
                }
                "AUTH" => self.auth(arg).await?,
                "MAIL" => self.mail(arg).await?,
                "RCPT" => self.rcpt(arg).await?,
                "DATA" => self.data().await?,
                "RSET" => {
                    self.reset();
                    self.reply("250 2.0.0 OK").await?;
                }
                "NOOP" => self.reply("250 2.0.0 OK").await?,
                "VRFY" => self.reply("252 2.1.5 Cannot verify, will accept").await?,
                "QUIT" => return self.reply("221 2.0.0 Bye").await,
                "STARTTLS" | "EXPN" | "TURN" | "BDAT" => {
                    self.reply("502 5.5.1 Command not implemented").await?;
                }
                _ => self.reply("500 5.5.2 Command not recognized").await?,
            }
        }
    }

    async fn ehlo(&mut self, client: &str) -> io::Result<()> {
        self.reset();
        let reply = format!(
            "250-{host} greets {client}\r\n\
             250-SIZE {size}\r\n\
             250-8BITMIME\r\n\
             250-SMTPUTF8\r\n\
             250-PIPELINING\r\n\
             250-AUTH PLAIN LOGIN\r\n\
             250 ENHANCEDSTATUSCODES",
            host = self.config.hostname,
            client = client.trim(),
            size = self.config.max_message_bytes,
        );
        self.reply(&reply).await
    }

    /// Accepts any credentials; the exchange only has to be completed. The username
    /// is kept for the rest of the session and can select a project.
    async fn auth(&mut self, arg: &str) -> io::Result<()> {
        let mut parts = arg.split_whitespace();
        let mechanism = parts.next().unwrap_or_default().to_ascii_uppercase();
        let initial_response = parts.next().map(str::to_owned);
        let username = match mechanism.as_str() {
            "PLAIN" => {
                let response = match initial_response {
                    Some(response) => response,
                    None => match self.challenge("334 ").await? {
                        Some(response) => response,
                        None => return Ok(()),
                    },
                };
                plain_username(&response)
            }
            "LOGIN" => {
                // Base64 for "Username:" and "Password:".
                let user = match initial_response {
                    Some(response) => response,
                    None => match self.challenge("334 VXNlcm5hbWU6").await? {
                        Some(response) => response,
                        None => return Ok(()),
                    },
                };
                // The password is read to complete the exchange and then dropped.
                if self.challenge("334 UGFzc3dvcmQ6").await?.is_none() {
                    return Ok(());
                }
                decode_text(&user)
            }
            _ => {
                return self
                    .reply("504 5.5.4 Unrecognized authentication type")
                    .await;
            }
        };
        self.auth_user = username.as_deref().and_then(AuthUser::parse);
        self.reply("235 2.7.0 Authentication successful").await
    }

    /// Sends an AUTH prompt and returns the client's answer. `None` when the exchange
    /// ended instead (cancelled, line too long or connection closed); any reply for that
    /// has been sent already.
    async fn challenge(&mut self, prompt: &str) -> io::Result<Option<String>> {
        self.reply(prompt).await?;
        match self.next_line(MAX_COMMAND_LINE).await? {
            Line::Complete if self.line.trim_ascii() == b"*" => {
                self.reply("501 5.7.0 Authentication cancelled").await?;
                Ok(None)
            }
            Line::Complete => Ok(Some(String::from_utf8_lossy(&self.line).trim().to_owned())),
            Line::TooLong => {
                self.reply("500 5.5.6 Line too long").await?;
                Ok(None)
            }
            Line::Closed => Ok(None),
        }
    }

    async fn mail(&mut self, arg: &str) -> io::Result<()> {
        let Some((address, params)) = path_argument(arg, "FROM:") else {
            return self.reply("501 5.5.4 Syntax: MAIL FROM:<address>").await;
        };
        let declared_size = params.split_whitespace().find_map(|param| {
            param
                .to_ascii_uppercase()
                .strip_prefix("SIZE=")?
                .parse::<usize>()
                .ok()
        });
        if declared_size.is_some_and(|size| size > self.config.max_message_bytes) {
            self.messages.stats().record_rejected();
            return self.reply("552 5.3.4 Message size exceeds limit").await;
        }
        if self.config.capture.is_paused() {
            info!(from = address, "message refused: capture is paused");
            return self
                .reply("451 4.3.2 Mailbeaver is paused, try again later")
                .await;
        }
        self.reset();
        self.sender = Some(address.to_owned());
        self.reply("250 2.1.0 OK").await
    }

    async fn rcpt(&mut self, arg: &str) -> io::Result<()> {
        if self.sender.is_none() {
            return self.reply("503 5.5.1 Need MAIL before RCPT").await;
        }
        let Some((address, _)) = path_argument(arg, "TO:") else {
            return self.reply("501 5.5.4 Syntax: RCPT TO:<address>").await;
        };
        if address.is_empty() {
            return self.reply("501 5.1.3 Empty recipient").await;
        }
        if self.recipients.len() >= MAX_RECIPIENTS {
            return self.reply("452 4.5.3 Too many recipients").await;
        }
        self.recipients.push(address.to_owned());
        self.reply("250 2.1.5 OK").await
    }

    async fn data(&mut self) -> io::Result<()> {
        let Some(sender) = self.sender.clone() else {
            return self.reply("503 5.5.1 Need MAIL before DATA").await;
        };
        if self.recipients.is_empty() {
            return self.reply("503 5.5.1 Need RCPT before DATA").await;
        }
        self.reply("354 End data with <CR><LF>.<CR><LF>").await?;

        let limit = self.config.max_message_bytes;
        let mut message = Vec::new();
        let mut too_large = false;
        loop {
            // One line may not exceed the whole budget; a longer one cannot be stored anyway.
            match self.next_line(limit.saturating_add(2)).await? {
                Line::Complete => {}
                Line::TooLong => {
                    self.messages.stats().record_rejected();
                    return self.reply("552 5.3.4 Message size exceeds limit").await;
                }
                Line::Closed => return Ok(()),
            }
            if self.line == b".\r\n" || self.line == b".\n" {
                break;
            }
            if too_large {
                continue;
            }
            let content = self.line.strip_prefix(b".").unwrap_or(&self.line);
            let content = content
                .strip_suffix(b"\r\n")
                .or_else(|| content.strip_suffix(b"\n"))
                .unwrap_or(content);
            if message.len() + content.len() + 2 > limit {
                too_large = true;
                continue;
            }
            message.extend_from_slice(content);
            message.extend_from_slice(b"\r\n");
        }

        let envelope = Envelope {
            mail_from: sender,
            rcpt_to: std::mem::take(&mut self.recipients),
            auth_user: self.auth_user.clone(),
        };
        self.reset();
        // Envelope addresses help to find out why mail is missing; content never.
        let from = envelope.mail_from.clone();
        let recipients = envelope.rcpt_to.join(", ");
        let user = envelope
            .auth_user
            .as_ref()
            .map_or_else(|| "-".to_owned(), |user| user.as_str().to_owned());
        if too_large {
            self.messages.stats().record_rejected();
            warn!(%from, to = %recipients, "message rejected: larger than the limit");
            return self.reply("552 5.3.4 Message size exceeds limit").await;
        }
        match self.messages.receive(envelope, message).await {
            Ok(id) => {
                info!(id = id.0, %from, to = %recipients, %user, "message received");
                self.reply(&format!("250 2.0.0 OK: queued as {}", id.0))
                    .await
            }
            Err(err) => {
                self.messages.stats().record_rejected();
                warn!(error = %err, %from, "storing message failed");
                self.reply("451 4.3.0 Message could not be stored").await
            }
        }
    }

    fn reset(&mut self) {
        self.sender = None;
        self.recipients.clear();
    }

    async fn reply(&mut self, text: &str) -> io::Result<()> {
        self.writer.write_all(text.as_bytes()).await?;
        self.writer.write_all(b"\r\n").await?;
        self.writer.flush().await
    }

    /// Reads one line including its terminator into `self.line`.
    async fn next_line(&mut self, max: usize) -> io::Result<Line> {
        self.line.clear();
        let limit = u64::try_from(max).unwrap_or(u64::MAX).saturating_add(1);
        let read = read_until_newline(&mut self.reader, &mut self.line, limit);
        let Ok(read) = tokio::time::timeout(IDLE_TIMEOUT, read).await else {
            self.reply("421 4.4.2 Idle timeout, closing connection")
                .await?;
            return Ok(Line::Closed);
        };
        let count = read?;
        Ok(if count == 0 {
            Line::Closed
        } else if self.line.ends_with(b"\n") {
            Line::Complete
        } else if self.line.len() > max {
            Line::TooLong
        } else {
            // Peer closed mid-line.
            Line::Closed
        })
    }
}

async fn read_until_newline<R>(reader: &mut R, buf: &mut Vec<u8>, limit: u64) -> io::Result<usize>
where
    R: AsyncBufRead + Unpin,
{
    reader.take(limit).read_until(b'\n', buf).await
}

/// Decodes a base64 AUTH response into text; `None` for invalid base64 or UTF-8.
fn decode_text(response: &str) -> Option<String> {
    let bytes = BASE64.decode(response.trim()).ok()?;
    String::from_utf8(bytes).ok()
}

/// The authentication identity of a SASL PLAIN response (`authzid NUL authcid NUL
/// password`, RFC 4616).
fn plain_username(response: &str) -> Option<String> {
    let decoded = decode_text(response)?;
    decoded.split('\0').nth(1).map(str::to_owned)
}

/// Parses `FROM:<addr> PARAMS` / `TO:<addr>`; angle brackets are optional for lenient clients.
fn path_argument<'a>(arg: &'a str, keyword: &str) -> Option<(&'a str, &'a str)> {
    let head = arg.get(..keyword.len())?;
    if !head.eq_ignore_ascii_case(keyword) {
        return None;
    }
    let rest = arg.get(keyword.len()..)?.trim_start();
    if let Some(inner) = rest.strip_prefix('<') {
        let (address, params) = inner.split_once('>')?;
        Some((address.trim(), params.trim()))
    } else {
        let (address, params) = rest.split_once(' ').unwrap_or((rest, ""));
        Some((address.trim(), params.trim()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::changes::ChangeFeed;
    use crate::db::Database;
    use crate::messages::{ListQuery, MessageId};
    use tokio::io::duplex;

    fn config(max_message_bytes: usize) -> SmtpConfig {
        SmtpConfig {
            hostname: "test".into(),
            max_message_bytes,
            max_connections: 1,
            capture: Capture::new(ChangeFeed::new()),
        }
    }

    /// Sends a scripted client conversation and returns the server output.
    async fn converse(messages: &MessageService, max_message_bytes: usize, input: &str) -> String {
        let (mut client, server) = duplex(64 * 1024);
        client.write_all(input.as_bytes()).await.unwrap();
        client.shutdown().await.unwrap();
        handle(server, messages, &config(max_message_bytes))
            .await
            .unwrap();
        let mut output = String::new();
        client.read_to_string(&mut output).await.unwrap();
        output
    }

    fn service() -> MessageService {
        MessageService::new(
            Database::open_in_memory().unwrap(),
            ChangeFeed::new(),
            crate::stats::Stats::new(),
            100,
        )
    }

    #[tokio::test]
    async fn accepts_pipelined_message_with_auth() {
        let messages = service();
        let output = converse(
            &messages,
            1024,
            "EHLO client\r\nAUTH PLAIN AHVzZXIAcGFzcw==\r\nMAIL FROM:<a@example.com> SIZE=100\r\n\
             RCPT TO:<b@example.com>\r\nDATA\r\nSubject: Hi\r\n\r\n..dot\r\nbody\r\n.\r\nQUIT\r\n",
        )
        .await;
        assert!(output.contains("235 2.7.0"), "{output}");
        assert!(output.contains("250 2.0.0 OK: queued as 1"), "{output}");
        assert!(output.ends_with("221 2.0.0 Bye\r\n"), "{output}");

        let raw = messages.raw(MessageId(1)).await.unwrap().unwrap();
        assert_eq!(raw, b"Subject: Hi\r\n\r\n.dot\r\nbody\r\n");
        let detail = messages.detail(MessageId(1)).await.unwrap().unwrap();
        assert_eq!(detail.envelope_from, "a@example.com");
        assert_eq!(detail.envelope_to, ["b@example.com"]);
        // "AHVzZXIAcGFzcw==" is "\0user\0pass"; the unknown user gets its own project.
        assert_eq!(detail.auth_user.as_deref(), Some("user"));
        assert_eq!(
            detail.summary.project.map(|p| p.name).as_deref(),
            Some("user")
        );
    }

    #[tokio::test]
    async fn auth_login_username_applies_to_the_rest_of_the_session() {
        let messages = service();
        let message = "MAIL FROM:<a@x>\r\nRCPT TO:<b@x>\r\nDATA\r\nSubject: x\r\n\r\nbody\r\n.\r\n";
        // "c2hvcA==" is "shop".
        let input =
            format!("EHLO c\r\n{message}AUTH LOGIN c2hvcA==\r\ncGFzcw==\r\nRSET\r\n{message}");
        let output = converse(&messages, 1024, &input).await;
        assert_eq!(
            output.matches("250 2.0.0 OK: queued").count(),
            2,
            "{output}"
        );

        let first = messages.detail(MessageId(1)).await.unwrap().unwrap();
        let second = messages.detail(MessageId(2)).await.unwrap().unwrap();
        assert_eq!(first.auth_user, None);
        assert_eq!(second.auth_user.as_deref(), Some("shop"));
    }

    #[tokio::test]
    async fn auth_login_prompts_for_both_values() {
        let output = converse(
            &service(),
            1024,
            "EHLO c\r\nAUTH LOGIN\r\ndXNlcg==\r\ncGFzcw==\r\n",
        )
        .await;
        assert!(
            output.contains("334 VXNlcm5hbWU6\r\n334 UGFzc3dvcmQ6\r\n235"),
            "{output}"
        );
    }

    #[tokio::test]
    async fn enforces_command_order() {
        let output = converse(&service(), 1024, "EHLO c\r\nRCPT TO:<b@x>\r\nDATA\r\n").await;
        assert!(
            output.contains("503 5.5.1 Need MAIL before RCPT"),
            "{output}"
        );
        assert!(
            output.contains("503 5.5.1 Need MAIL before DATA"),
            "{output}"
        );
    }

    #[tokio::test]
    async fn rejects_oversized_message_and_stays_usable() {
        let messages = service();
        let body = "x".repeat(60);
        let input = format!(
            "MAIL FROM:<a@x>\r\nRCPT TO:<b@x>\r\nDATA\r\n{body}\r\n{body}\r\n.\r\n\
             MAIL FROM:<a@x> SIZE=500\r\nNOOP\r\n"
        );
        let output = converse(&messages, 100, &input).await;
        assert_eq!(output.matches("552 5.3.4").count(), 2, "{output}");
        let db = Database::open_in_memory().unwrap();
        assert_eq!(
            messages.stats().snapshot(&db).await.unwrap().smtp_rejected,
            2
        );
        assert!(output.contains("250 2.0.0 OK\r\n"), "{output}");
        assert_eq!(messages.list(ListQuery::default()).await.unwrap().total, 0);
    }

    #[test]
    fn decodes_auth_usernames() {
        assert_eq!(plain_username("AHVzZXIAcGFzcw==").as_deref(), Some("user"));
        assert_eq!(
            plain_username("YWRtaW4AdXNlcgBwYXNz").as_deref(),
            Some("user")
        );
        assert_eq!(plain_username("not base64!"), None);
        assert_eq!(decode_text("c2hvcA==").as_deref(), Some("shop"));
        assert_eq!(decode_text("/w=="), None);
    }

    #[test]
    fn parses_path_arguments() {
        assert_eq!(
            path_argument("FROM:<a@x> SIZE=1", "FROM:"),
            Some(("a@x", "SIZE=1"))
        );
        assert_eq!(path_argument("to: <b@x>", "TO:"), Some(("b@x", "")));
        assert_eq!(path_argument("FROM:<>", "FROM:"), Some(("", "")));
        assert_eq!(path_argument("FROM:a@x", "FROM:"), Some(("a@x", "")));
        assert_eq!(path_argument("TO:<b@x>", "FROM:"), None);
    }

    #[tokio::test]
    async fn refuses_mail_while_paused() {
        let messages = service();
        let config = config(1024);
        config.capture.set_paused(true);
        let (mut client, server) = duplex(64 * 1024);
        client
            .write_all(b"EHLO x\r\nMAIL FROM:<a@x.test>\r\nQUIT\r\n")
            .await
            .unwrap();
        client.shutdown().await.unwrap();
        handle(server, &messages, &config).await.unwrap();
        let mut output = String::new();
        client.read_to_string(&mut output).await.unwrap();
        assert!(
            output.contains("451 4.3.2 Mailbeaver is paused"),
            "{output}"
        );
    }
}
