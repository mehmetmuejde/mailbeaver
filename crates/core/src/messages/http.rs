//! HTTP endpoints for messages, folders and the trash.

use axum::Json;
use axum::extract::{FromRef, Path, Query, State};
use axum::http::header::{
    CONTENT_DISPOSITION, CONTENT_SECURITY_POLICY, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS,
};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::http::{ApiError, ErrorBody};
use crate::projects::ProjectId;

use super::domain::{
    DeleteOutcome, Folder, FolderCounts, ListQuery, MessageDetail, MessageId, MessagePage,
    ProjectFilter,
};
use super::search::SearchQuery;
use super::service::MessageService;

const DEFAULT_PAGE_SIZE: u32 = 50;
const MAX_PAGE_SIZE: u32 = 200;
const MAX_SEARCH_CHARS: usize = 256;
/// Value of the `project` parameter that selects messages without a project.
const UNASSIGNED: &str = "none";

/// Downloads must never execute as a page on the API origin.
const DOWNLOAD_CSP: &str = "sandbox";

pub fn router<S>() -> OpenApiRouter<S>
where
    S: Clone + Send + Sync + 'static,
    MessageService: FromRef<S>,
{
    // One `routes!` per path: utoipa-axum mounts all handlers of a call on each path.
    OpenApiRouter::default()
        .routes(routes!(list_messages, clear_messages))
        .routes(routes!(message_detail, delete_message))
        .routes(routes!(mark_read))
        .routes(routes!(mark_unread))
        .routes(routes!(move_message))
        // A JSON body forces a CORS preflight, so foreign pages cannot send it.
        .routes(routes!(mark_all_read))
        .routes(routes!(raw_message))
        .routes(routes!(download_attachment))
        .routes(routes!(folder_counts))
        .routes(routes!(empty_trash))
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct ListParams {
    /// Absent lists every folder.
    folder: Option<Folder>,
    /// Absent lists every project, `none` the unassigned messages, otherwise a project id.
    project: Option<String>,
    /// Full-text search over subject, sender, recipients and body; every word must match
    /// as a prefix. Filters: `from:`, `to:`, `subject:`, `body:`, `is:unread`,
    /// `is:read`, `has:attachment`, `project:<name>` or `project:none`,
    /// `after:YYYY-MM-DD`, `before:YYYY-MM-DD` (UTC days). Quote values with spaces.
    q: Option<String>,
    /// Unix milliseconds: only messages received at or after this time.
    since: Option<i64>,
    /// Page size, 1 to 200 (default 50).
    limit: Option<u32>,
    /// Messages to skip, newest first.
    offset: Option<u32>,
}

impl TryFrom<ListParams> for ListQuery {
    type Error = ApiError;

    fn try_from(params: ListParams) -> Result<Self, Self::Error> {
        let project = match params.project.as_deref() {
            None => ProjectFilter::All,
            Some(UNASSIGNED) => ProjectFilter::Unassigned,
            Some(id) => ProjectFilter::Project(ProjectId(
                id.parse()
                    .map_err(|_| ApiError::BadRequest("invalid project".to_owned()))?,
            )),
        };
        let search = match params.q.as_deref() {
            Some(q) if q.chars().count() > MAX_SEARCH_CHARS => {
                return Err(ApiError::BadRequest("search query too long".to_owned()));
            }
            Some(q) => SearchQuery::parse(q),
            None => None,
        };
        Ok(Self {
            folder: params.folder,
            exclude_trash: false,
            project,
            search,
            since: params.since,
            limit: params
                .limit
                .unwrap_or(DEFAULT_PAGE_SIZE)
                .clamp(1, MAX_PAGE_SIZE),
            offset: params.offset.unwrap_or(0),
        })
    }
}

#[utoipa::path(get, path = "/api/messages", tag = "messages",
    params(ListParams),
    responses(
        (status = 200, description = "Messages newest first, with totals of the whole query", body = MessagePage),
        (status = 400, description = "Invalid filter", body = ErrorBody),
    ))]
async fn list_messages(
    State(messages): State<MessageService>,
    Query(params): Query<ListParams>,
) -> Result<Json<MessagePage>, ApiError> {
    Ok(Json(messages.list(params.try_into()?).await?))
}

#[utoipa::path(get, path = "/api/folders", tag = "messages",
    responses((status = 200, description = "Total and unread messages per folder", body = FolderCounts)))]
async fn folder_counts(
    State(messages): State<MessageService>,
) -> Result<Json<FolderCounts>, ApiError> {
    Ok(Json(messages.folder_counts().await?))
}

#[utoipa::path(delete, path = "/api/messages", tag = "messages",
    responses((status = 204, description = "Every message in every folder was deleted")))]
async fn clear_messages(State(messages): State<MessageService>) -> Result<StatusCode, ApiError> {
    messages.clear().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/api/messages/{id}", tag = "messages",
    params(("id" = i64, Path, description = "Message id")),
    responses(
        (status = 200, description = "The message with headers, bodies and attachment list", body = MessageDetail),
        (status = 404, description = "No such message", body = ErrorBody),
    ))]
async fn message_detail(
    State(messages): State<MessageService>,
    Path(id): Path<i64>,
) -> Result<Json<MessageDetail>, ApiError> {
    let detail = messages
        .detail(MessageId(id))
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(detail))
}

/// Permanent delete; messages outside Trash have to be moved there first.
#[utoipa::path(delete, path = "/api/messages/{id}", tag = "messages",
    params(("id" = i64, Path, description = "Message id")),
    responses(
        (status = 204, description = "Deleted for good"),
        (status = 404, description = "No such message", body = ErrorBody),
        (status = 409, description = "The message is not in Trash", body = ErrorBody),
    ))]
async fn delete_message(
    State(messages): State<MessageService>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    match messages.delete(MessageId(id)).await? {
        DeleteOutcome::Deleted => Ok(StatusCode::NO_CONTENT),
        DeleteOutcome::NotFound => Err(ApiError::NotFound),
        DeleteOutcome::NotInTrash => Err(ApiError::Conflict(
            "only messages in trash can be deleted permanently".to_owned(),
        )),
    }
}

#[utoipa::path(post, path = "/api/messages/{id}/read", tag = "messages",
    params(("id" = i64, Path, description = "Message id")),
    responses(
        (status = 204, description = "Marked read"),
        (status = 404, description = "No such message", body = ErrorBody),
    ))]
async fn mark_read(
    State(messages): State<MessageService>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    if messages.mark_read(MessageId(id)).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

#[utoipa::path(post, path = "/api/messages/{id}/unread", tag = "messages",
    params(("id" = i64, Path, description = "Message id")),
    responses(
        (status = 204, description = "Marked unread"),
        (status = 404, description = "No such message", body = ErrorBody),
    ))]
async fn mark_unread(
    State(messages): State<MessageService>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    if messages.mark_unread(MessageId(id)).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
struct MoveRequest {
    folder: Folder,
}

#[utoipa::path(post, path = "/api/messages/{id}/move", tag = "messages",
    params(("id" = i64, Path, description = "Message id")),
    request_body = MoveRequest,
    responses(
        (status = 204, description = "Moved; entering Trash records the time"),
        (status = 404, description = "No such message", body = ErrorBody),
    ))]
async fn move_message(
    State(messages): State<MessageService>,
    Path(id): Path<i64>,
    Json(request): Json<MoveRequest>,
) -> Result<StatusCode, ApiError> {
    if messages.move_to(MessageId(id), request.folder).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
struct MarkAllReadRequest {
    /// `None` marks every folder.
    folder: Option<Folder>,
}

#[derive(Debug, Serialize, ToSchema)]
struct CountResponse {
    count: usize,
}

#[utoipa::path(post, path = "/api/messages/mark-read", tag = "messages",
    request_body = MarkAllReadRequest,
    responses((status = 200, description = "How many messages changed", body = CountResponse)))]
async fn mark_all_read(
    State(messages): State<MessageService>,
    Json(request): Json<MarkAllReadRequest>,
) -> Result<Json<CountResponse>, ApiError> {
    let count = messages.mark_all_read(request.folder).await?;
    Ok(Json(CountResponse { count }))
}

#[utoipa::path(delete, path = "/api/trash", tag = "messages",
    responses((status = 204, description = "Trash was emptied")))]
async fn empty_trash(State(messages): State<MessageService>) -> Result<StatusCode, ApiError> {
    messages.empty_trash().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct RawParams {
    /// `true` sends the message as an `.eml` file named after its subject.
    download: Option<bool>,
}

#[utoipa::path(get, path = "/api/messages/{id}/raw", tag = "messages",
    params(("id" = i64, Path, description = "Message id"), RawParams),
    responses(
        (status = 200, description = "The message as received (RFC 5322); `message/rfc822` with `download=true`", body = String, content_type = "text/plain"),
        (status = 404, description = "No such message", body = ErrorBody),
    ))]
async fn raw_message(
    State(messages): State<MessageService>,
    Path(id): Path<i64>,
    Query(params): Query<RawParams>,
) -> Result<Response, ApiError> {
    let id = MessageId(id);
    let raw = messages.raw(id).await?.ok_or(ApiError::NotFound)?;
    let security = [
        (X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")),
        (
            CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(DOWNLOAD_CSP),
        ),
    ];
    if params.download != Some(true) {
        let content_type = (
            CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        return Ok((security, [content_type], raw).into_response());
    }
    let subject = messages
        .detail(id)
        .await?
        .map(|detail| detail.summary.subject)
        .unwrap_or_default();
    let disposition = HeaderValue::from_str(&eml_disposition(&subject, id))
        .unwrap_or(HeaderValue::from_static("attachment"));
    Ok((
        security,
        [
            (CONTENT_TYPE, HeaderValue::from_static("message/rfc822")),
            (CONTENT_DISPOSITION, disposition),
        ],
        raw,
    )
        .into_response())
}

const MAX_FILENAME_CHARS: usize = 80;

/// `attachment` with an ASCII `filename` for old clients and the full subject as
/// RFC 5987 `filename*`, so umlauts survive.
fn eml_disposition(subject: &str, id: MessageId) -> String {
    let name: String = subject
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                ' '
            } else {
                c
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(MAX_FILENAME_CHARS)
        .collect();
    let name = if name.is_empty() {
        format!("message-{}", id.0)
    } else {
        name
    };
    let ascii: String = name
        .chars()
        .map(|c| if c.is_ascii() { c } else { '_' })
        .collect();
    let encoded: String = format!("{name}.eml")
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
                char::from(byte).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect();
    format!("attachment; filename=\"{ascii}.eml\"; filename*=UTF-8''{encoded}")
}

#[utoipa::path(get, path = "/api/messages/{id}/attachments/{index}", tag = "messages",
    params(("id" = i64, Path, description = "Message id"), ("index" = usize, Path, description = "Position in the attachment list")),
    responses(
        (status = 200, description = "The attachment as a download", body = Vec<u8>, content_type = "application/octet-stream"),
        (status = 404, description = "No such message or attachment", body = ErrorBody),
    ))]
async fn download_attachment(
    State(messages): State<MessageService>,
    Path((id, index)): Path<(i64, usize)>,
) -> Result<Response, ApiError> {
    let attachment = messages
        .attachment(MessageId(id), index)
        .await?
        .ok_or(ApiError::NotFound)?;
    let content_type = HeaderValue::from_str(&attachment.content_type)
        .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
    let disposition = HeaderValue::from_str(&format!(
        "attachment; filename=\"{}\"",
        safe_filename(&attachment.filename)
    ))
    .unwrap_or_else(|_| HeaderValue::from_static("attachment"));
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

/// Keeps filenames header-safe: ASCII letters, digits, dot, dash, underscore.
fn safe_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filenames_are_header_safe() {
        assert_eq!(
            safe_filename("Rechnung März \"1\".pdf"),
            "Rechnung_M_rz__1_.pdf"
        );
    }

    #[test]
    fn names_eml_files_after_the_subject() {
        assert_eq!(
            eml_disposition("Bestellbestätigung: Nr. 12/3", MessageId(4)),
            "attachment; filename=\"Bestellbest_tigung Nr. 12 3.eml\"; \
             filename*=UTF-8''Bestellbest%C3%A4tigung%20Nr.%2012%203.eml"
        );
        assert_eq!(
            eml_disposition("  \"\" ", MessageId(7)),
            "attachment; filename=\"message-7.eml\"; filename*=UTF-8''message-7.eml"
        );
        let long = eml_disposition(&"x".repeat(200), MessageId(1));
        assert!(long.contains(&format!("\"{}.eml\"", "x".repeat(MAX_FILENAME_CHARS))));
    }
}
