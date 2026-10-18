//! HTTP endpoints for managing projects.

use axum::Json;
use axum::extract::{FromRef, Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::clock::now_millis;
use crate::http::{ApiError, ErrorBody};

use super::domain::{Project, ProjectId, ProjectName, Rule, RuleKind, RuleSuggestion};
use super::service::{ProjectError, ProjectInput, ProjectOverview, ProjectService, dedupe_rules};

pub fn router<S>() -> OpenApiRouter<S>
where
    S: Clone + Send + Sync + 'static,
    ProjectService: FromRef<S>,
{
    OpenApiRouter::default()
        .routes(routes!(overview, create))
        .routes(routes!(update, remove))
        .routes(routes!(rule_suggestions))
}

const MAX_PREFIX_CHARS: usize = 254;

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct SuggestionParams {
    kind: RuleKind,
    /// Start of the value typed so far; empty lists the most frequent ones.
    q: Option<String>,
}

#[utoipa::path(get, path = "/api/projects/rule-suggestions", tag = "projects", params(SuggestionParams),
    responses((status = 200, description = "Up to 20 values from captured mail, most frequent first", body = [RuleSuggestion])))]
async fn rule_suggestions(
    State(projects): State<ProjectService>,
    Query(params): Query<SuggestionParams>,
) -> Result<Json<Vec<RuleSuggestion>>, ApiError> {
    let prefix: String = params
        .q
        .unwrap_or_default()
        .trim()
        .chars()
        .take(MAX_PREFIX_CHARS)
        .collect();
    Ok(Json(projects.rule_suggestions(params.kind, prefix).await?))
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
struct RuleRequest {
    kind: RuleKind,
    value: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
struct ProjectRequest {
    name: String,
    rules: Vec<RuleRequest>,
    /// Defaults to `true`.
    #[serde(default = "notify_by_default")]
    notify: bool,
}

fn notify_by_default() -> bool {
    true
}

impl TryFrom<ProjectRequest> for ProjectInput {
    type Error = ApiError;

    fn try_from(request: ProjectRequest) -> Result<Self, Self::Error> {
        let name = ProjectName::parse(&request.name)
            .map_err(|err| ApiError::Unprocessable(err.to_string()))?;
        let rules = request
            .rules
            .iter()
            .map(|rule| Rule::parse(rule.kind, &rule.value))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| ApiError::Unprocessable(err.to_string()))?;
        Ok(Self {
            name,
            rules: dedupe_rules(rules),
            notify: request.notify,
        })
    }
}

impl From<ProjectError> for ApiError {
    fn from(err: ProjectError) -> Self {
        match err {
            ProjectError::NotFound => Self::NotFound,
            ProjectError::NameTaken(_) | ProjectError::RuleTaken { .. } => {
                Self::Conflict(err.to_string())
            }
            ProjectError::TooManyRules => Self::Unprocessable(err.to_string()),
            ProjectError::Db(err) => Self::Internal(err),
        }
    }
}

#[utoipa::path(get, path = "/api/projects", tag = "projects",
    responses((status = 200, description = "Projects with their rules and message counts", body = ProjectOverview)))]
async fn overview(
    State(projects): State<ProjectService>,
) -> Result<Json<ProjectOverview>, ApiError> {
    Ok(Json(projects.overview().await?))
}

#[utoipa::path(post, path = "/api/projects", tag = "projects",
    request_body = ProjectRequest,
    responses(
        (status = 201, description = "Created; existing mail is assigned again", body = Project),
        (status = 409, description = "Name or rule already used by another project", body = ErrorBody),
        (status = 422, description = "Invalid name or rule", body = ErrorBody),
    ))]
async fn create(
    State(projects): State<ProjectService>,
    Json(request): Json<ProjectRequest>,
) -> Result<(StatusCode, Json<Project>), ApiError> {
    let project = projects.create(request.try_into()?, now_millis()).await?;
    Ok((StatusCode::CREATED, Json(project)))
}

#[utoipa::path(put, path = "/api/projects/{id}", tag = "projects",
    params(("id" = i64, Path, description = "Project id")),
    request_body = ProjectRequest,
    responses(
        (status = 200, description = "Updated; the rules replace the old ones", body = Project),
        (status = 404, description = "No such project", body = ErrorBody),
        (status = 409, description = "Name or rule already used by another project", body = ErrorBody),
        (status = 422, description = "Invalid name or rule", body = ErrorBody),
    ))]
async fn update(
    State(projects): State<ProjectService>,
    Path(id): Path<i64>,
    Json(request): Json<ProjectRequest>,
) -> Result<Json<Project>, ApiError> {
    Ok(Json(
        projects.update(ProjectId(id), request.try_into()?).await?,
    ))
}

#[utoipa::path(delete, path = "/api/projects/{id}", tag = "projects",
    params(("id" = i64, Path, description = "Project id")),
    responses(
        (status = 204, description = "Deleted; its messages are assigned again"),
        (status = 404, description = "No such project", body = ErrorBody),
    ))]
async fn remove(
    State(projects): State<ProjectService>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    projects.delete(ProjectId(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}
