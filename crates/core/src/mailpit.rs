//! Mailpit-compatible API (ADR-0029): the part of Mailpit's `/api/v1` that test helpers
//! use, so tools written for Mailpit work against Mailbeaver unchanged.
//!
//! Mailpit has no folders: these endpoints see every message outside Trash. Deleting is
//! permanent, as in Mailpit. Projects show up as tags.

use std::collections::BTreeMap;

use axum::Json;
use axum::extract::{FromRef, Path, Query, State};
use axum::http::HeaderValue;
use axum::http::StatusCode;
use axum::http::header::{
    CONTENT_DISPOSITION, CONTENT_SECURITY_POLICY, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS,
};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::http::{ApiError, ErrorBody};
use crate::messages::{
    DeleteOutcome, Folder, ListQuery, MessageDetail, MessageId, MessageService, MessageSummary,
    SearchQuery,
};

const DEFAULT_LIMIT: u32 = 50;
const MAX_LIMIT: u32 = 1000;
/// Bulk changes (delete or mark a whole search) go through at most this many messages.
const BULK_LIMIT: u32 = 100_000;
const LATEST: &str = "latest";
const MILLIS_PER_DAY: i64 = 24 * 60 * 60 * 1000;
/// Downloads must never execute as a page on the API origin.
const DOWNLOAD_CSP: &str = "sandbox";

pub fn router<S>() -> OpenApiRouter<S>
where
    S: Clone + Send + Sync + 'static,
    MessageService: FromRef<S>,
{
    OpenApiRouter::default()
        .routes(routes!(list_messages, delete_messages, set_read))
        .routes(routes!(search, delete_search))
        .routes(routes!(message))
        .routes(routes!(raw))
        .routes(routes!(headers))
        .routes(routes!(part))
        .routes(routes!(info))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "PascalCase")]
struct Address {
    name: String,
    address: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "PascalCase")]
struct MessageItem {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "MessageID")]
    message_id: String,
    read: bool,
    from: Address,
    to: Vec<Address>,
    cc: Vec<Address>,
    bcc: Vec<Address>,
    reply_to: Vec<Address>,
    subject: String,
    /// RFC 3339, when Mailbeaver received the message.
    created: String,
    tags: Vec<String>,
    size: i64,
    /// Number of attachments.
    attachments: usize,
    snippet: String,
}

#[derive(Debug, Serialize, ToSchema)]
struct MessagesPage {
    /// Messages outside Trash.
    total: i64,
    unread: i64,
    /// Messages on this page.
    count: usize,
    /// Messages matching the request, across all pages.
    messages_count: i64,
    start: u32,
    tags: Vec<String>,
    messages: Vec<MessageItem>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "PascalCase")]
struct Part {
    #[serde(rename = "PartID")]
    part_id: String,
    file_name: String,
    content_type: String,
    #[serde(rename = "ContentID")]
    content_id: String,
    size: usize,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "PascalCase")]
struct Message {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "MessageID")]
    message_id: String,
    from: Address,
    to: Vec<Address>,
    cc: Vec<Address>,
    bcc: Vec<Address>,
    reply_to: Vec<Address>,
    return_path: String,
    subject: String,
    /// RFC 3339, when Mailbeaver received the message.
    date: String,
    tags: Vec<String>,
    /// SMTP username, empty without authentication.
    username: String,
    text: String,
    #[serde(rename = "HTML")]
    html: String,
    size: i64,
    inline: Vec<Part>,
    attachments: Vec<Part>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "PascalCase")]
struct Info {
    version: String,
    messages: i64,
    unread: i64,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct PageParams {
    /// Offset of the first message, default 0.
    start: Option<u32>,
    /// Messages per page, default 50, at most 1000.
    limit: Option<u32>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct SearchParams {
    /// Mailbeaver's search syntax (ADR-0027), e.g. `from:shop.test is:unread`.
    query: String,
    start: Option<u32>,
    limit: Option<u32>,
}

#[derive(Debug, Default, Deserialize, ToSchema)]
#[serde(rename_all = "PascalCase")]
struct IdsRequest {
    /// Message ids; empty or missing means every message.
    #[serde(rename = "IDs", default)]
    ids: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "PascalCase")]
struct ReadRequest {
    /// Message ids; empty or missing means every message.
    #[serde(rename = "IDs", default)]
    ids: Vec<String>,
    read: bool,
}

#[utoipa::path(get, path = "/api/v1/messages", tag = "mailpit", params(PageParams),
    responses((status = 200, description = "Newest messages outside Trash", body = MessagesPage)))]
async fn list_messages(
    State(messages): State<MessageService>,
    Query(params): Query<PageParams>,
) -> Result<Json<MessagesPage>, ApiError> {
    Ok(Json(
        page(&messages, None, params.start, params.limit).await?,
    ))
}

#[utoipa::path(get, path = "/api/v1/search", tag = "mailpit", params(SearchParams),
    responses(
        (status = 200, description = "Messages outside Trash matching the query", body = MessagesPage),
        (status = 400, description = "Empty query", body = ErrorBody),
    ))]
async fn search(
    State(messages): State<MessageService>,
    Query(params): Query<SearchParams>,
) -> Result<Json<MessagesPage>, ApiError> {
    let query = search_query(&params.query)?;
    Ok(Json(
        page(&messages, Some(query), params.start, params.limit).await?,
    ))
}

#[utoipa::path(delete, path = "/api/v1/messages", tag = "mailpit", request_body = IdsRequest,
    responses(
        (status = 200, description = "Deleted for good; without ids every message, Trash included"),
        (status = 404, description = "An id does not exist", body = ErrorBody),
    ))]
async fn delete_messages(
    State(messages): State<MessageService>,
    body: Option<Json<IdsRequest>>,
) -> Result<StatusCode, ApiError> {
    let ids = parse_ids(&body.map(|Json(body)| body).unwrap_or_default().ids)?;
    if ids.is_empty() {
        messages.clear().await?;
        return Ok(StatusCode::OK);
    }
    for id in ids {
        delete_permanently(&messages, id).await?;
    }
    Ok(StatusCode::OK)
}

#[utoipa::path(delete, path = "/api/v1/search", tag = "mailpit", params(SearchParams),
    responses(
        (status = 200, description = "Every message outside Trash matching the query was deleted for good"),
        (status = 400, description = "Empty query", body = ErrorBody),
    ))]
async fn delete_search(
    State(messages): State<MessageService>,
    Query(params): Query<SearchParams>,
) -> Result<StatusCode, ApiError> {
    let query = search_query(&params.query)?;
    for id in all_ids(&messages, Some(query)).await? {
        delete_permanently(&messages, id).await?;
    }
    Ok(StatusCode::OK)
}

#[utoipa::path(put, path = "/api/v1/messages", tag = "mailpit", request_body = ReadRequest,
    responses(
        (status = 200, description = "Read state set"),
        (status = 404, description = "An id does not exist", body = ErrorBody),
    ))]
async fn set_read(
    State(messages): State<MessageService>,
    Json(request): Json<ReadRequest>,
) -> Result<StatusCode, ApiError> {
    let mut ids = parse_ids(&request.ids)?;
    if ids.is_empty() {
        if request.read {
            messages.mark_all_read(None).await?;
            return Ok(StatusCode::OK);
        }
        ids = all_ids(&messages, None).await?;
    }
    for id in ids {
        let found = if request.read {
            messages.mark_read(id).await?
        } else {
            messages.mark_unread(id).await?
        };
        if !found {
            return Err(ApiError::NotFound);
        }
    }
    Ok(StatusCode::OK)
}

#[utoipa::path(get, path = "/api/v1/message/{id}", tag = "mailpit",
    params(("id" = String, Path, description = "Message id, or `latest`")),
    responses(
        (status = 200, description = "The message; reading it marks it read, as in Mailpit", body = Message),
        (status = 404, description = "No such message", body = ErrorBody),
    ))]
async fn message(
    State(messages): State<MessageService>,
    Path(id): Path<String>,
) -> Result<Json<Message>, ApiError> {
    let id = resolve(&messages, &id).await?;
    let detail = messages.detail(id).await?.ok_or(ApiError::NotFound)?;
    messages.mark_read(id).await?;
    Ok(Json(to_message(detail)))
}

#[utoipa::path(get, path = "/api/v1/message/{id}/raw", tag = "mailpit",
    params(("id" = String, Path, description = "Message id, or `latest`")),
    responses(
        (status = 200, description = "The message as received", body = String, content_type = "text/plain"),
        (status = 404, description = "No such message", body = ErrorBody),
    ))]
async fn raw(
    State(messages): State<MessageService>,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let id = resolve(&messages, &id).await?;
    let raw = messages.raw(id).await?.ok_or(ApiError::NotFound)?;
    Ok((
        [
            (
                CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            ),
            (X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")),
            (
                CONTENT_SECURITY_POLICY,
                HeaderValue::from_static(DOWNLOAD_CSP),
            ),
        ],
        raw,
    )
        .into_response())
}

#[utoipa::path(get, path = "/api/v1/message/{id}/headers", tag = "mailpit",
    params(("id" = String, Path, description = "Message id, or `latest`")),
    responses(
        (status = 200, description = "Every header with its values", body = BTreeMap<String, Vec<String>>),
        (status = 404, description = "No such message", body = ErrorBody),
    ))]
async fn headers(
    State(messages): State<MessageService>,
    Path(id): Path<String>,
) -> Result<Json<BTreeMap<String, Vec<String>>>, ApiError> {
    let id = resolve(&messages, &id).await?;
    let detail = messages.detail(id).await?.ok_or(ApiError::NotFound)?;
    let mut headers: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for line in detail.body.headers {
        headers.entry(line.name).or_default().push(line.value);
    }
    Ok(Json(headers))
}

#[utoipa::path(get, path = "/api/v1/message/{id}/part/{part}", tag = "mailpit",
    params(
        ("id" = String, Path, description = "Message id, or `latest`"),
        ("part" = String, Path, description = "`PartID` of an attachment"),
    ),
    responses(
        (status = 200, description = "The attachment", content_type = "application/octet-stream"),
        (status = 404, description = "No such message or part", body = ErrorBody),
    ))]
async fn part(
    State(messages): State<MessageService>,
    Path((id, part)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let id = resolve(&messages, &id).await?;
    let index = part.parse().map_err(|_| ApiError::NotFound)?;
    let attachment = messages
        .attachment(id, index)
        .await?
        .ok_or(ApiError::NotFound)?;
    let content_type = HeaderValue::from_str(&attachment.content_type)
        .unwrap_or(HeaderValue::from_static("application/octet-stream"));
    // Only the characters a quoted filename can carry safely.
    let filename: String = attachment
        .filename
        .chars()
        .filter(|c| (c.is_ascii_graphic() && *c != '"' && *c != '\\') || *c == ' ')
        .collect();
    let disposition = HeaderValue::from_str(&format!("attachment; filename=\"{filename}\""))
        .unwrap_or(HeaderValue::from_static("attachment"));
    Ok((
        [
            (CONTENT_TYPE, content_type),
            (CONTENT_DISPOSITION, disposition),
            (X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")),
            (
                CONTENT_SECURITY_POLICY,
                HeaderValue::from_static(DOWNLOAD_CSP),
            ),
        ],
        attachment.data,
    )
        .into_response())
}

#[utoipa::path(get, path = "/api/v1/info", tag = "mailpit",
    responses((status = 200, description = "Version and message counts", body = Info)))]
async fn info(State(messages): State<MessageService>) -> Result<Json<Info>, ApiError> {
    let counts = messages.folder_counts().await?;
    Ok(Json(Info {
        version: env!("CARGO_PKG_VERSION").to_owned(),
        messages: counts.inbox.total + counts.archive.total,
        unread: counts.inbox.unread + counts.archive.unread,
    }))
}

async fn page(
    messages: &MessageService,
    search: Option<SearchQuery>,
    start: Option<u32>,
    limit: Option<u32>,
) -> Result<MessagesPage, ApiError> {
    let start = start.unwrap_or(0);
    let found = messages
        .list(ListQuery {
            exclude_trash: true,
            search,
            limit: limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT),
            offset: start,
            ..ListQuery::default()
        })
        .await?;
    let counts = messages.folder_counts().await?;
    let items: Vec<MessageItem> = found.messages.into_iter().map(to_item).collect();
    Ok(MessagesPage {
        total: counts.inbox.total + counts.archive.total,
        unread: counts.inbox.unread + counts.archive.unread,
        count: items.len(),
        messages_count: found.total,
        start,
        tags: Vec::new(),
        messages: items,
    })
}

fn search_query(input: &str) -> Result<SearchQuery, ApiError> {
    SearchQuery::parse(input).ok_or_else(|| ApiError::BadRequest("empty search".to_owned()))
}

async fn all_ids(
    messages: &MessageService,
    search: Option<SearchQuery>,
) -> Result<Vec<MessageId>, ApiError> {
    let found = messages
        .list(ListQuery {
            exclude_trash: true,
            search,
            limit: BULK_LIMIT,
            ..ListQuery::default()
        })
        .await?;
    Ok(found
        .messages
        .into_iter()
        .map(|message| message.id)
        .collect())
}

/// Mailbeaver deletes for good only from Trash, so the message goes there first.
async fn delete_permanently(messages: &MessageService, id: MessageId) -> Result<(), ApiError> {
    if !messages.move_to(id, Folder::Trash).await? {
        return Err(ApiError::NotFound);
    }
    match messages.delete(id).await? {
        DeleteOutcome::Deleted => Ok(()),
        DeleteOutcome::NotFound | DeleteOutcome::NotInTrash => Err(ApiError::NotFound),
    }
}

fn parse_ids(ids: &[String]) -> Result<Vec<MessageId>, ApiError> {
    ids.iter()
        .map(|id| {
            id.parse()
                .map(MessageId)
                .map_err(|_| ApiError::BadRequest(format!("invalid message id {id:?}")))
        })
        .collect()
}

/// A numeric id, or `latest` for the newest message outside Trash.
async fn resolve(messages: &MessageService, id: &str) -> Result<MessageId, ApiError> {
    if id != LATEST {
        return id.parse().map(MessageId).map_err(|_| ApiError::NotFound);
    }
    let newest = messages
        .list(ListQuery {
            exclude_trash: true,
            limit: 1,
            ..ListQuery::default()
        })
        .await?;
    newest
        .messages
        .first()
        .map(|message| message.id)
        .ok_or(ApiError::NotFound)
}

fn to_item(summary: MessageSummary) -> MessageItem {
    MessageItem {
        id: summary.id.0.to_string(),
        message_id: String::new(),
        read: summary.is_read,
        from: address(&summary.from),
        to: summary.to.iter().map(|to| address(to)).collect(),
        cc: Vec::new(),
        bcc: Vec::new(),
        reply_to: Vec::new(),
        created: rfc3339(summary.received_at),
        tags: summary
            .project
            .map(|project| project.name)
            .into_iter()
            .collect(),
        size: summary.size,
        attachments: usize::from(summary.has_attachments),
        snippet: summary.preview,
        subject: summary.subject,
    }
}

fn to_message(detail: MessageDetail) -> Message {
    let header = |name: &str| {
        detail
            .body
            .headers
            .iter()
            .find(|line| line.name.eq_ignore_ascii_case(name))
            .map(|line| line.value.clone())
            .unwrap_or_default()
    };
    let message_id = header("Message-ID")
        .trim()
        .trim_start_matches('<')
        .trim_end_matches('>')
        .to_owned();
    let (cc, reply_to) = (addresses(&header("Cc")), addresses(&header("Reply-To")));
    let summary = detail.summary;
    Message {
        id: summary.id.0.to_string(),
        message_id,
        from: address(&summary.from),
        to: summary.to.iter().map(|to| address(to)).collect(),
        cc,
        // Bcc is not part of a received message.
        bcc: Vec::new(),
        reply_to,
        return_path: detail.envelope_from,
        date: rfc3339(summary.received_at),
        tags: summary
            .project
            .map(|project| project.name)
            .into_iter()
            .collect(),
        username: detail.auth_user.unwrap_or_default(),
        text: detail.body.text.unwrap_or_default(),
        html: detail.body.html.unwrap_or_default(),
        size: summary.size,
        inline: Vec::new(),
        attachments: detail
            .body
            .attachments
            .into_iter()
            .map(|attachment| Part {
                part_id: attachment.index.to_string(),
                file_name: attachment.filename,
                content_type: attachment.content_type,
                content_id: String::new(),
                size: attachment.size,
            })
            .collect(),
        subject: summary.subject,
    }
}

/// `Name <address>` or a bare address.
fn address(value: &str) -> Address {
    let value = value.trim();
    match value.rsplit_once('<') {
        Some((name, rest)) if rest.ends_with('>') => Address {
            name: name.trim().trim_matches('"').to_owned(),
            address: rest.trim_end_matches('>').trim().to_owned(),
        },
        _ => Address {
            name: String::new(),
            address: value.to_owned(),
        },
    }
}

/// A header list of addresses; commas inside quotes or angle brackets do not split.
fn addresses(value: &str) -> Vec<Address> {
    let mut parts = Vec::new();
    let (mut current, mut quoted, mut angled) = (String::new(), false, false);
    for c in value.chars() {
        match c {
            '"' => quoted = !quoted,
            '<' if !quoted => angled = true,
            '>' if !quoted => angled = false,
            ',' if !quoted && !angled => {
                parts.push(std::mem::take(&mut current));
                continue;
            }
            _ => {}
        }
        current.push(c);
    }
    parts.push(current);
    parts
        .iter()
        .filter(|part| !part.trim().is_empty())
        .map(|part| address(part))
        .collect()
}

/// Unix milliseconds as `2026-10-17T08:30:00.000Z`.
fn rfc3339(millis: i64) -> String {
    let days = millis.div_euclid(MILLIS_PER_DAY);
    let of_day = millis.rem_euclid(MILLIS_PER_DAY);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        of_day / 3_600_000,
        of_day / 60_000 % 60,
        of_day / 1000 % 60,
        of_day % 1000
    )
}

/// Date of a day count since 1970-01-01 (Howard Hinnant's `civil_from_days`).
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let mp = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_times_as_rfc3339() {
        assert_eq!(rfc3339(0), "1970-01-01T00:00:00.000Z");
        assert_eq!(rfc3339(1_792_195_200_000), "2026-10-17T00:00:00.000Z");
        assert_eq!(
            rfc3339(1_709_164_800_000 + 45_296_789),
            "2024-02-29T12:34:56.789Z"
        );
    }

    #[test]
    fn splits_names_and_addresses() {
        assert_eq!(
            address("Shop Team <orders@shop.test>"),
            Address {
                name: "Shop Team".into(),
                address: "orders@shop.test".into()
            }
        );
        assert_eq!(address("dev@team.test").address, "dev@team.test");
        let list = addresses(r#""Doe, Jane" <jane@x.test>, bob@x.test"#);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "Doe, Jane");
        assert_eq!(list[1].address, "bob@x.test");
        assert!(addresses("").is_empty());
    }
}
