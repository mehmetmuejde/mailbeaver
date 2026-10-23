use axum::Json;
use axum::extract::{FromRef, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::clock::now_millis;
use crate::http::{ApiError, ErrorBody};

use super::domain::{Level, LogEntry, LogQuery, NewLogEntry, Source};
use super::service::{LogError, LogService};

const DEFAULT_LIMIT: u32 = 200;
const MAX_LIMIT: u32 = 1000;

pub fn router<S>() -> OpenApiRouter<S>
where
    S: Clone + Send + Sync + 'static,
    LogService: FromRef<S>,
{
    OpenApiRouter::default().routes(routes!(list, record, clear))
}

#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
struct ListParams {
    /// Only entries at least this severe; `info` (everything) when left out.
    min_level: Option<Level>,
    /// `backend` or `frontend`; both when left out.
    source: Option<Source>,
    /// Newest entries returned, default 200, at most 1000.
    limit: Option<u32>,
}

/// An entry of the UI.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
struct UiLogRequest {
    level: Level,
    /// UI area, e.g. `query` or `window`.
    target: String,
    message: String,
}

impl From<LogError> for ApiError {
    fn from(err: LogError) -> Self {
        match err {
            LogError::RateLimited => Self::TooManyRequests(err.to_string()),
            LogError::Db(err) => Self::Internal(err),
        }
    }
}

#[utoipa::path(get, path = "/api/logs", tag = "logs", params(ListParams),
    responses((status = 200, description = "Newest log entries first", body = [LogEntry])))]
async fn list(
    State(logs): State<LogService>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<LogEntry>>, ApiError> {
    let query = LogQuery {
        min_level: params.min_level.unwrap_or(Level::Info),
        source: params.source,
        limit: params.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT),
    };
    Ok(Json(logs.list(query).await?))
}

#[utoipa::path(post, path = "/api/logs", tag = "logs", request_body = UiLogRequest,
    responses(
        (status = 204, description = "Stored as a `frontend` entry"),
        (status = 429, description = "The UI sent too many entries this minute", body = ErrorBody),
    ))]
async fn record(
    State(logs): State<LogService>,
    Json(request): Json<UiLogRequest>,
) -> Result<StatusCode, ApiError> {
    let entry = NewLogEntry::new(
        now_millis(),
        request.level,
        Source::Frontend,
        &request.target,
        &request.message,
    );
    logs.record_from_ui(entry).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(delete, path = "/api/logs", tag = "logs",
    responses((status = 204, description = "Every log entry was deleted")))]
async fn clear(State(logs): State<LogService>) -> Result<StatusCode, ApiError> {
    logs.clear().await?;
    Ok(StatusCode::NO_CONTENT)
}
