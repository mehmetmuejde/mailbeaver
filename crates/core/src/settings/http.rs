//! HTTP endpoints for the settings.

use axum::Json;
use axum::extract::{FromRef, State};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::http::{ApiError, ErrorBody};

use axum::extract::Path;

use crate::messages::Folder;

use super::domain::{FolderView, FolderViews, ServerSettings, Settings};
use super::service::SettingsService;

pub fn router<S>() -> OpenApiRouter<S>
where
    S: Clone + Send + Sync + 'static,
    SettingsService: FromRef<S>,
{
    OpenApiRouter::default()
        .routes(routes!(read, update))
        .routes(routes!(folder_views))
        .routes(routes!(save_folder_view))
        .routes(routes!(read_server, save_server))
}

#[utoipa::path(get, path = "/api/server-settings", tag = "settings",
    responses((status = 200, description = "Ports and limits saved in the app; `null` keeps the default", body = ServerSettings)))]
async fn read_server(
    State(settings): State<SettingsService>,
) -> Result<Json<ServerSettings>, ApiError> {
    Ok(Json(settings.server().await?))
}

#[utoipa::path(put, path = "/api/server-settings", tag = "settings",
    request_body = ServerSettings,
    responses(
        (status = 200, description = "Saved; applies at the next start unless an environment variable sets the value", body = ServerSettings),
        (status = 422, description = "Invalid value", body = ErrorBody),
    ))]
async fn save_server(
    State(settings): State<SettingsService>,
    Json(server): Json<ServerSettings>,
) -> Result<Json<ServerSettings>, ApiError> {
    server
        .validate()
        .map_err(|err| ApiError::Unprocessable(err.to_string()))?;
    Ok(Json(settings.save_server(server).await?))
}

#[utoipa::path(get, path = "/api/settings", tag = "settings",
    responses((status = 200, description = "The current settings", body = Settings)))]
async fn read(State(settings): State<SettingsService>) -> Result<Json<Settings>, ApiError> {
    Ok(Json(settings.get().await?))
}

#[utoipa::path(put, path = "/api/settings", tag = "settings",
    request_body = Settings,
    responses(
        (status = 200, description = "Saved; a shorter retention empties expired Trash at once", body = Settings),
        (status = 422, description = "Invalid value", body = ErrorBody),
    ))]
async fn update(
    State(settings): State<SettingsService>,
    Json(new): Json<Settings>,
) -> Result<Json<Settings>, ApiError> {
    Ok(Json(settings.update(new).await?))
}

#[utoipa::path(get, path = "/api/folder-views", tag = "settings",
    responses((status = 200, description = "How each folder was last shown", body = FolderViews)))]
async fn folder_views(
    State(settings): State<SettingsService>,
) -> Result<Json<FolderViews>, ApiError> {
    Ok(Json(settings.folder_views().await?))
}

#[utoipa::path(put, path = "/api/folder-views/{folder}", tag = "settings",
    params(("folder" = Folder, Path, description = "inbox, archive or trash")),
    request_body = FolderView,
    responses(
        (status = 200, description = "Saved", body = FolderView),
        (status = 422, description = "Invalid value", body = ErrorBody),
    ))]
async fn save_folder_view(
    State(settings): State<SettingsService>,
    Path(folder): Path<Folder>,
    Json(view): Json<FolderView>,
) -> Result<Json<FolderView>, ApiError> {
    Ok(Json(settings.save_folder_view(folder, view).await?))
}
