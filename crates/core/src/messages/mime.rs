//! Turns raw RFC 5322 messages into the views the API serves.
//!
//! Parsing happens on demand from the stored raw bytes; only the list, search and
//! project columns are extracted once at receive time.

use mail_parser::{Addr, Address, Message, MessageParser, MimeHeaders};
use serde::Serialize;
use utoipa::ToSchema;

/// Upper bound for the text kept for search; a 25 MB body adds little beyond this.
const MAX_SEARCH_TEXT_BYTES: usize = 1024 * 1024;

/// Fields extracted once when a message arrives.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Summary {
    pub from: String,
    /// The bare address of the first `From`, empty when there is none.
    pub from_address: String,
    pub to: Vec<String>,
    pub subject: String,
    pub has_attachments: bool,
    /// Plain text of the body (the HTML part converted when there is no text part).
    pub body_text: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HeaderLine {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentInfo {
    pub index: usize,
    pub filename: String,
    pub content_type: String,
    pub size: usize,
}

/// The parsed body of a message: headers, bodies and attachment metadata.
#[derive(Debug, Clone, Default, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ParsedBody {
    pub headers: Vec<HeaderLine>,
    pub text: Option<String>,
    pub html: Option<String>,
    pub attachments: Vec<AttachmentInfo>,
}

#[derive(Debug, Clone)]
pub struct Attachment {
    pub filename: String,
    pub content_type: String,
    pub data: Vec<u8>,
}

fn parse(raw: &[u8]) -> Option<Message<'_>> {
    MessageParser::new().parse(raw)
}

/// Unparseable messages still get stored, with empty fields.
pub fn summarize(raw: &[u8]) -> Summary {
    let Some(message) = parse(raw) else {
        return Summary::default();
    };
    let first_from = message.from().and_then(Address::first);
    Summary {
        from: first_from.map(format_addr).unwrap_or_default(),
        from_address: first_from
            .and_then(Addr::address)
            .unwrap_or_default()
            .to_owned(),
        to: message
            .to()
            .map(|to| to.iter().map(format_addr).collect())
            .unwrap_or_default(),
        subject: message.subject().unwrap_or_default().to_owned(),
        has_attachments: message.attachments().next().is_some(),
        body_text: message
            .body_text(0)
            .map(|text| truncate_at_char_boundary(&text, MAX_SEARCH_TEXT_BYTES).to_owned())
            .unwrap_or_default(),
    }
}

fn truncate_at_char_boundary(text: &str, max_bytes: usize) -> &str {
    if text.len() <= max_bytes {
        return text;
    }
    let end = (0..=max_bytes)
        .rev()
        .find(|&index| text.is_char_boundary(index))
        .unwrap_or_default();
    text.get(..end).unwrap_or_default()
}

pub fn parse_body(raw: &[u8]) -> ParsedBody {
    let Some(message) = parse(raw) else {
        return ParsedBody::default();
    };

    let headers = message
        .headers()
        .iter()
        .map(|header| HeaderLine {
            name: header.name().to_owned(),
            value: raw_header_value(raw, header.offset_start(), header.offset_end()),
        })
        .collect();

    // body_text/body_html convert between formats when one is missing; the UI should
    // only show what the sender actually sent.
    let html = message
        .html_bodies()
        .find(|part| part.is_text_html())
        .and_then(|part| part.text_contents())
        .map(str::to_owned);
    let text = message
        .text_bodies()
        .find(|part| part.is_text() && !part.is_text_html())
        .and_then(|part| part.text_contents())
        .map(str::to_owned);

    let attachments = message
        .attachments()
        .enumerate()
        .map(|(index, part)| AttachmentInfo {
            index,
            filename: attachment_filename(part.attachment_name(), index),
            content_type: content_type(part),
            size: part.len(),
        })
        .collect();

    ParsedBody {
        headers,
        text,
        html,
        attachments,
    }
}

pub fn attachment(raw: &[u8], index: usize) -> Option<Attachment> {
    let message = parse(raw)?;
    let part = message.attachments().nth(index)?;
    Some(Attachment {
        filename: attachment_filename(part.attachment_name(), index),
        content_type: content_type(part),
        data: part.contents().to_vec(),
    })
}

fn format_addr(addr: &Addr<'_>) -> String {
    match (addr.name(), addr.address()) {
        (Some(name), Some(address)) => format!("{name} <{address}>"),
        (None, Some(address)) => address.to_owned(),
        (Some(name), None) => name.to_owned(),
        (None, None) => String::new(),
    }
}

/// Header value as sent, with folded lines joined.
fn raw_header_value(raw: &[u8], start: u32, end: u32) -> String {
    let (Ok(start), Ok(end)) = (usize::try_from(start), usize::try_from(end)) else {
        return String::new();
    };
    raw.get(start..end)
        .map(|bytes| {
            String::from_utf8_lossy(bytes)
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default()
}

fn content_type<'x>(part: &impl MimeHeaders<'x>) -> String {
    part.content_type()
        .map(|ct| match ct.subtype() {
            Some(subtype) => format!("{}/{subtype}", ct.ctype()),
            None => ct.ctype().to_owned(),
        })
        .unwrap_or_else(|| "application/octet-stream".to_owned())
}

fn attachment_filename(name: Option<&str>, index: usize) -> String {
    name.filter(|name| !name.trim().is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("attachment-{}", index + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    // `\x20` keeps the folding space; a literal space after a `\` line continuation is stripped.
    const MULTIPART: &[u8] = b"From: Alice <alice@example.com>\r\n\
To: Bob <bob@example.com>, carol@example.com\r\n\
Subject: Hello\r\n\
\x20World\r\n\
MIME-Version: 1.0\r\n\
Content-Type: multipart/mixed; boundary=\"b1\"\r\n\
\r\n\
--b1\r\n\
Content-Type: multipart/alternative; boundary=\"b2\"\r\n\
\r\n\
--b2\r\n\
Content-Type: text/plain\r\n\
\r\n\
Plain body\r\n\
--b2\r\n\
Content-Type: text/html\r\n\
\r\n\
<p>HTML body</p>\r\n\
--b2--\r\n\
--b1\r\n\
Content-Type: application/pdf\r\n\
Content-Disposition: attachment; filename=\"invoice.pdf\"\r\n\
Content-Transfer-Encoding: base64\r\n\
\r\n\
JVBERi0=\r\n\
--b1--\r\n";

    #[test]
    fn summarize_extracts_list_fields() {
        let summary = summarize(MULTIPART);
        assert_eq!(summary.from, "Alice <alice@example.com>");
        assert_eq!(summary.from_address, "alice@example.com");
        assert_eq!(summary.to, ["Bob <bob@example.com>", "carol@example.com"]);
        assert_eq!(summary.subject, "Hello World");
        assert!(summary.has_attachments);
        assert_eq!(summary.body_text.trim(), "Plain body");
    }

    #[test]
    fn summarize_converts_html_only_bodies_to_text() {
        let summary =
            summarize(b"Content-Type: text/html\r\n\r\n<p>Reset your <b>password</b></p>\r\n");
        assert!(
            summary.body_text.contains("Reset your password"),
            "{summary:?}"
        );
    }

    #[test]
    fn truncates_on_char_boundaries() {
        assert_eq!(truncate_at_char_boundary("äöü", 3), "ä");
        assert_eq!(truncate_at_char_boundary("abc", 10), "abc");
    }

    #[test]
    fn parse_body_keeps_original_parts() {
        let body = parse_body(MULTIPART);
        assert_eq!(body.text.as_deref().map(str::trim), Some("Plain body"));
        assert_eq!(
            body.html.as_deref().map(str::trim),
            Some("<p>HTML body</p>")
        );
        assert_eq!(body.attachments.len(), 1);
        assert_eq!(body.attachments[0].filename, "invoice.pdf");
        assert_eq!(body.attachments[0].content_type, "application/pdf");
        let subject = body.headers.iter().find(|h| h.name == "Subject");
        assert_eq!(subject.map(|h| h.value.as_str()), Some("Hello World"));
    }

    #[test]
    fn text_only_message_has_no_html() {
        let body = parse_body(b"Subject: Hi\r\n\r\nJust text\r\n");
        assert!(body.html.is_none());
        assert_eq!(body.text.as_deref().map(str::trim), Some("Just text"));
    }

    #[test]
    fn attachment_returns_decoded_bytes() {
        let file = attachment(MULTIPART, 0);
        assert_eq!(
            file.as_ref().map(|f| f.data.as_slice()),
            Some(&b"%PDF-"[..])
        );
        assert!(attachment(MULTIPART, 1).is_none());
    }
}
