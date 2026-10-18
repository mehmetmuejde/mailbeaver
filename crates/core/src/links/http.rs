//! HTTP endpoint for checking links.

use axum::Json;
use axum::extract::{FromRef, State};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::http::{ApiError, ErrorBody};

use super::checker::{LinkCheckError, LinkChecker, LinkStatus};

pub fn router<S>() -> OpenApiRouter<S>
where
    S: Clone + Send + Sync + 'static,
    LinkChecker: FromRef<S>,
{
    // A JSON body forces a CORS preflight, so foreign pages cannot make Mailbeaver send
    // requests on their behalf.
    OpenApiRouter::default().routes(routes!(check_links))
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
struct CheckRequest {
    /// Up to 50 http or https URLs.
    urls: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
struct CheckResponse {
    results: Vec<LinkStatus>,
}

impl From<LinkCheckError> for ApiError {
    fn from(err: LinkCheckError) -> Self {
        Self::Unprocessable(err.to_string())
    }
}

#[utoipa::path(post, path = "/api/links/check", tag = "links",
    request_body = CheckRequest,
    responses(
        (status = 200, description = "Status per URL, in request order", body = CheckResponse),
        (status = 422, description = "Too many URLs or not http(s)", body = ErrorBody),
    ))]
async fn check_links(
    State(checker): State<LinkChecker>,
    Json(request): Json<CheckRequest>,
) -> Result<Json<CheckResponse>, ApiError> {
    let results = checker.check_all(request.urls).await?;
    Ok(Json(CheckResponse { results }))
}
