//! HTTP API for the UI: assembles the feature routers and adds what all of them share —
//! error responses, the `Host` check, CORS, the change stream and server information.

use std::convert::Infallible;
use std::sync::Arc;

use axum::extract::{FromRef, Request, State};
use axum::http::header::{CONTENT_TYPE, HOST};
use axum::http::{HeaderValue, Method, StatusCode};
use axum::middleware::{self, Next};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::error;

use crate::capture::Capture;
use crate::changes::ChangeFeed;
use crate::db::{Database, DbError};
use crate::links::{self, LinkChecker};
use crate::logs::{self, LogService};
use crate::maintenance::MaintenanceService;
use crate::messages::{self, MessageService};
use crate::projects::{self, ProjectService};
use crate::settings::{self, SettingsService};
use crate::stats::{Stats, StatsSnapshot};
use crate::updates::{UpdateChecker, UpdateStatus};
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub struct ApiConfig {
    /// Origins of the UI (Tauri webview, Vite dev server) allowed to call the API.
    pub allowed_origins: Vec<HeaderValue>,
    /// Accepted `Host` header values; `None` disables the check (non-loopback bind).
    pub allowed_hosts: Option<Vec<String>>,
    pub info: ServerInfo,
    /// Answers every other path, e.g. the built UI of the headless server.
    pub ui: Option<Router>,
}

/// Answer of the health check.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Health {
    /// Always `ok` while the server answers.
    pub status: &'static str,
}

/// What the settings page shows about the running servers.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    /// App version from `Cargo.toml`.
    pub version: String,
    pub smtp_address: String,
    pub http_address: String,
    pub database: String,
    pub max_messages: u32,
    pub max_message_bytes: usize,
    /// Server settings set by an environment variable (`smtpPort`, `httpPort`,
    /// `maxMessages`, `maxMessageMb`); the app cannot change them.
    pub from_environment: Vec<String>,
    /// `desktop` or `server`: what can restart the servers.
    pub host: String,
}

/// The services every handler can extract.
#[derive(Clone)]
pub struct Services {
    pub messages: MessageService,
    pub projects: ProjectService,
    pub settings: SettingsService,
    pub maintenance: MaintenanceService,
    pub stats: Stats,
    pub links: LinkChecker,
    pub logs: LogService,
    /// For figures that need the database itself, e.g. its size.
    pub db: Database,
    pub changes: ChangeFeed,
    pub capture: Capture,
    pub updates: UpdateChecker,
}

#[derive(Clone)]
pub struct AppState {
    services: Services,
    info: Arc<ServerInfo>,
}

/// The error body every failing endpoint returns.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    /// What went wrong, safe to show to users.
    pub error: String,
}

impl FromRef<AppState> for MessageService {
    fn from_ref(state: &AppState) -> Self {
        state.services.messages.clone()
    }
}

impl FromRef<AppState> for ProjectService {
    fn from_ref(state: &AppState) -> Self {
        state.services.projects.clone()
    }
}

impl FromRef<AppState> for LogService {
    fn from_ref(state: &AppState) -> Self {
        state.services.logs.clone()
    }
}

impl FromRef<AppState> for LinkChecker {
    fn from_ref(state: &AppState) -> Self {
        state.services.links.clone()
    }
}

impl FromRef<AppState> for SettingsService {
    fn from_ref(state: &AppState) -> Self {
        state.services.settings.clone()
    }
}

/// Info and tags of the generated OpenAPI document; the paths come from the handlers.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Mailbeaver API",
        description = "Read, search and manage the mail Mailbeaver caught. The desktop UI uses the same API.",
        license(name = "MIT", identifier = "MIT"),
    ),
    tags(
        (name = "messages", description = "Captured mail, folders, search and Trash"),
        (name = "projects", description = "Projects and the rules that assign mail to them"),
        (name = "settings", description = "User settings"),
        (name = "links", description = "Checking the links of a message"),
        (name = "server", description = "Server information, statistics and live changes"),
        (name = "logs", description = "Log of the backend and the UI"),
        (name = "mailpit", description = "Mailpit-compatible endpoints for existing test tools (ADR-0029)"),
    )
)]
struct ApiDoc;

/// Every documented endpoint; the OpenAPI document is generated from these handlers.
fn api_routes<S>() -> OpenApiRouter<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
    MessageService: FromRef<S>,
    ProjectService: FromRef<S>,
    SettingsService: FromRef<S>,
    LinkChecker: FromRef<S>,
    LogService: FromRef<S>,
{
    let mut doc = ApiDoc::openapi();
    doc.info.version = env!("CARGO_PKG_VERSION").to_owned();
    OpenApiRouter::with_openapi(doc)
        .merge(messages::router())
        .merge(projects::router())
        .merge(settings::router())
        .merge(links::router())
        .merge(logs::router())
        .merge(crate::mailpit::router())
        .routes(routes!(events))
        .routes(routes!(server_info))
        .routes(routes!(health))
        .routes(routes!(capture_state, set_capture))
        .routes(routes!(update_status))
        .routes(routes!(stats))
        // DELETE needs a CORS preflight, so no foreign page can trigger it with a form.
        .routes(routes!(reset_data))
}

pub fn router(services: Services, config: ApiConfig) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(config.allowed_origins))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([CONTENT_TYPE]);
    let state = AppState {
        services,
        info: Arc::new(config.info),
    };

    let (routes, spec) = api_routes::<AppState>().split_for_parts();
    let spec = Arc::new(spec);
    let mcp = crate::mcp::service(
        state.services.messages.clone(),
        state.services.changes.clone(),
    );
    let api = routes
        .nest_service("/mcp", mcp)
        .route(
            "/api/openapi.json",
            get(move || {
                let spec = Arc::clone(&spec);
                async move { Json(spec.as_ref().clone()) }
            }),
        )
        .with_state(state)
        .layer(cors);
    let api = match config.ui {
        Some(ui) => api.fallback_service(ui),
        None => api,
    };

    match config.allowed_hosts {
        Some(hosts) => api.layer(middleware::from_fn_with_state(
            Arc::<[String]>::from(hosts),
            require_known_host,
        )),
        None => api,
    }
}

/// Blocks DNS rebinding: a foreign site resolving its own name to 127.0.0.1 would
/// otherwise read every captured mail through the browser.
async fn require_known_host(
    State(hosts): State<Arc<[String]>>,
    request: Request,
    next: Next,
) -> Response {
    let host = request
        .headers()
        .get(HOST)
        .and_then(|value| value.to_str().ok());
    match host {
        Some(host) if hosts.iter().any(|allowed| allowed == host) => next.run(request).await,
        _ => StatusCode::MISDIRECTED_REQUEST.into_response(),
    }
}

/// Error responses shared by all features; each feature maps its own errors into it.
#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    NotFound,
    Conflict(String),
    Unprocessable(String),
    TooManyRequests(String),
    Internal(DbError),
}

impl From<DbError> for ApiError {
    fn from(err: DbError) -> Self {
        Self::Internal(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            Self::NotFound => (StatusCode::NOT_FOUND, "not found".to_owned()),
            Self::Conflict(message) => (StatusCode::CONFLICT, message),
            Self::Unprocessable(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            Self::TooManyRequests(message) => (StatusCode::TOO_MANY_REQUESTS, message),
            Self::Internal(err) => {
                // Logged once here, where the error is handled; the client gets no details.
                error!(error = %err, "api request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal error".to_owned(),
                )
            }
        };
        (status, Json(ErrorBody { error: message })).into_response()
    }
}

#[utoipa::path(delete, path = "/api/data", tag = "server",
    responses((status = 204, description = "Every message, project and setting was deleted")))]
async fn reset_data(State(state): State<AppState>) -> Result<StatusCode, ApiError> {
    state.services.maintenance.reset().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/api/stats", tag = "server",
    responses((status = 200, description = "Figures since the app started", body = StatsSnapshot)))]
async fn stats(State(state): State<AppState>) -> Result<Json<StatsSnapshot>, ApiError> {
    let services = &state.services;
    Ok(Json(services.stats.snapshot(&services.db).await?))
}

/// Whether the SMTP server currently refuses mail.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptureState {
    pub paused: bool,
}

#[utoipa::path(get, path = "/api/capture", tag = "server",
    responses((status = 200, description = "Whether capture is paused", body = CaptureState)))]
async fn capture_state(State(state): State<AppState>) -> Json<CaptureState> {
    Json(CaptureState {
        paused: state.services.capture.is_paused(),
    })
}

#[utoipa::path(put, path = "/api/capture", tag = "server", request_body = CaptureState,
    responses((status = 200, description = "Paused: SMTP answers 451 until resumed or restarted", body = CaptureState)))]
async fn set_capture(
    State(state): State<AppState>,
    Json(request): Json<CaptureState>,
) -> Json<CaptureState> {
    state.services.capture.set_paused(request.paused);
    Json(request)
}

#[utoipa::path(get, path = "/api/update", tag = "server",
    responses((status = 200, description = "The newest release, when the opt-in check found one", body = UpdateStatus)))]
async fn update_status(State(state): State<AppState>) -> Json<UpdateStatus> {
    Json(state.services.updates.status())
}

#[utoipa::path(get, path = "/api/health", tag = "server",
    responses((status = 200, description = "The server is up", body = Health)))]
async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}

#[utoipa::path(get, path = "/api/info", tag = "server",
    responses((status = 200, description = "Version, addresses and limits", body = ServerInfo)))]
async fn server_info(State(state): State<AppState>) -> Json<ServerInfo> {
    Json(state.info.as_ref().clone())
}

#[utoipa::path(get, path = "/api/events", tag = "server",
    responses((status = 200, description = "Server-Sent Events named `change`; the data is a JSON object with a `kind` such as `received`, `moved` or `reset`", content_type = "text/event-stream")))]
async fn events(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = BroadcastStream::new(state.services.changes.subscribe()).map(|change| {
        // A lagging client missed changes; telling it to resync is enough for a refresh.
        let data = change
            .ok()
            .and_then(|change| serde_json::to_string(&change).ok())
            .unwrap_or_else(|| r#"{"kind":"resync"}"#.to_owned());
        Ok(Event::default().event("change").data(data))
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    use super::*;
    use crate::db::Database;
    use crate::messages::{AuthUser, Envelope, Folder, ListQuery, MessageId};

    const HOST: &str = "127.0.0.1:8026";

    async fn serve() -> (SocketAddr, Services) {
        let db = Database::open_in_memory().unwrap();
        let changes = ChangeFeed::new();
        let services = Services {
            messages: MessageService::new(db.clone(), changes.clone(), Stats::new(), 10),
            projects: ProjectService::new(db.clone(), changes.clone()),
            settings: SettingsService::new(db.clone(), changes.clone()),
            maintenance: MaintenanceService::new(db.clone(), changes.clone()),
            stats: Stats::new(),
            links: LinkChecker::new().unwrap(),
            logs: LogService::new(db.clone()),
            capture: Capture::new(changes.clone()),
            updates: UpdateChecker::default(),
            db,
            changes,
        };
        services
            .messages
            .receive(
                Envelope {
                    mail_from: "orders@shop.example".into(),
                    ..Envelope::default()
                },
                b"From: Shop <orders@shop.example>\r\nSubject: Test\r\n\r\nBody\r\n".to_vec(),
            )
            .await
            .unwrap();
        let router = router(
            services.clone(),
            ApiConfig {
                allowed_origins: vec![HeaderValue::from_static("tauri://localhost")],
                allowed_hosts: Some(vec![HOST.into()]),
                info: ServerInfo {
                    version: "1.2.3".into(),
                    smtp_address: "127.0.0.1:1026".into(),
                    http_address: HOST.into(),
                    database: "/tmp/mailbeaver.db".into(),
                    max_messages: 10,
                    max_message_bytes: 1024,
                    from_environment: vec!["httpPort".into()],
                    host: "server".into(),
                },
                ui: Some(Router::new().route("/", get(|| async { "ui" }))),
            },
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, router).await });
        (addr, services)
    }

    async fn call(addr: SocketAddr, method: &str, path: &str) -> (u16, String) {
        send(addr, method, path, HOST, None).await
    }

    async fn call_json(addr: SocketAddr, method: &str, path: &str, json: &str) -> (u16, String) {
        send(addr, method, path, HOST, Some(json)).await
    }

    fn parse(body: &str) -> serde_json::Value {
        serde_json::from_str(body).unwrap()
    }

    /// Minimal HTTP/1.1 client so the tests need no extra dependency.
    async fn send(
        addr: SocketAddr,
        method: &str,
        path: &str,
        host: &str,
        json: Option<&str>,
    ) -> (u16, String) {
        let mut stream = TcpStream::connect(addr).await.unwrap();
        let body = match json {
            Some(json) => format!(
                "Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{json}",
                json.len()
            ),
            None => "\r\n".to_owned(),
        };
        let request =
            format!("{method} {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n{body}");
        stream.write_all(request.as_bytes()).await.unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).await.unwrap();
        let status = response[9..12].parse().unwrap();
        let body = response
            .split_once("\r\n\r\n")
            .map(|(_, body)| body.to_owned())
            .unwrap_or_default();
        (status, body)
    }

    #[tokio::test]
    async fn lists_and_reads_messages() {
        let (addr, _) = serve().await;
        let (status, body) = call(addr, "GET", "/api/messages").await;
        assert_eq!(status, 200);
        let page = parse(&body);
        assert_eq!(page["total"], 1);
        assert_eq!(page["messages"][0]["subject"], "Test");
        assert_eq!(page["messages"][0]["folder"], "inbox");

        let (_, body) = call(addr, "GET", "/api/messages/1").await;
        assert_eq!(parse(&body)["text"].as_str().map(str::trim), Some("Body"));
    }

    #[tokio::test]
    async fn rejects_foreign_host() {
        let (addr, _) = serve().await;
        let (status, _) = send(addr, "GET", "/api/messages", "evil.example:8026", None).await;
        assert_eq!(status, 421);
    }

    #[tokio::test]
    async fn deletes_permanently_only_from_trash() {
        let (addr, services) = serve().await;
        assert_eq!(call(addr, "DELETE", "/api/messages/1").await.0, 409);

        let (status, _) = call_json(
            addr,
            "POST",
            "/api/messages/1/move",
            r#"{"folder":"trash"}"#,
        )
        .await;
        assert_eq!(status, 204);
        let (_, body) = call(addr, "GET", "/api/folders").await;
        assert_eq!(parse(&body)["trash"]["total"], 1);

        assert_eq!(call(addr, "DELETE", "/api/messages/1").await.0, 204);
        assert_eq!(call(addr, "DELETE", "/api/messages/1").await.0, 404);
        assert_eq!(
            services
                .messages
                .list(ListQuery::default())
                .await
                .unwrap()
                .total,
            0
        );
    }

    #[tokio::test]
    async fn rejects_unknown_folders() {
        let (addr, _) = serve().await;
        let (status, _) =
            call_json(addr, "POST", "/api/messages/1/move", r#"{"folder":"spam"}"#).await;
        assert_eq!(status, 422);
        let (status, _) = call(addr, "GET", "/api/messages?folder=spam").await;
        assert_eq!(status, 400);
        let (status, _) = call_json(
            addr,
            "POST",
            "/api/messages/9/move",
            r#"{"folder":"inbox"}"#,
        )
        .await;
        assert_eq!(status, 404);
    }

    #[tokio::test]
    async fn filters_by_folder_search_and_project() {
        let (addr, _) = serve().await;
        let total = |path: &'static str| async move {
            let (status, body) = call(addr, "GET", path).await;
            assert_eq!(status, 200, "{path}: {body}");
            parse(&body)["total"].as_i64().unwrap()
        };
        assert_eq!(total("/api/messages?folder=inbox&q=bod").await, 1);
        assert_eq!(total("/api/messages?folder=archive").await, 0);
        assert_eq!(total("/api/messages?q=nothing").await, 0);
        assert_eq!(total("/api/messages?project=none").await, 1);
        assert_eq!(total("/api/messages?project=7").await, 0);

        let (status, _) = call(addr, "GET", "/api/messages?project=shop").await;
        assert_eq!(status, 400);
        let long = format!("/api/messages?q={}", "x".repeat(300));
        let (status, _) = send(addr, "GET", &long, HOST, None).await;
        assert_eq!(status, 400);
    }

    #[tokio::test]
    async fn empties_trash() {
        let (addr, services) = serve().await;
        services
            .messages
            .move_to(MessageId(1), Folder::Trash)
            .await
            .unwrap();
        let (status, _) = call(addr, "DELETE", "/api/trash").await;
        assert_eq!(status, 204);
        assert_eq!(
            services
                .messages
                .list(ListQuery::default())
                .await
                .unwrap()
                .total,
            0
        );
    }

    #[tokio::test]
    async fn manages_projects_and_assigns_existing_mail() {
        let (addr, services) = serve().await;
        let (status, body) = call_json(
            addr,
            "POST",
            "/api/projects",
            r#"{"name":"Shop","rules":[{"kind":"sender_domain","value":"Shop.Example"}]}"#,
        )
        .await;
        assert_eq!(status, 201, "{body}");
        let project = parse(&body);
        assert_eq!(project["rules"][0]["value"], "shop.example");
        let id = project["id"].as_i64().unwrap();

        let (_, body) = call(addr, "GET", "/api/projects").await;
        let overview = parse(&body);
        assert_eq!(overview["projects"][0]["name"], "Shop");
        assert_eq!(overview["projects"][0]["total"], 1);
        assert_eq!(overview["unassigned"]["total"], 0);

        let (_, body) = call(addr, "GET", "/api/messages/1").await;
        assert_eq!(parse(&body)["project"]["name"], "Shop");

        // Name and rules belong to one project only.
        let (status, body) = call_json(
            addr,
            "POST",
            "/api/projects",
            r#"{"name":"shop","rules":[]}"#,
        )
        .await;
        assert_eq!(status, 409, "{body}");
        let (status, body) = call_json(
            addr,
            "POST",
            "/api/projects",
            r#"{"name":"Other","rules":[{"kind":"sender_domain","value":"shop.example"}]}"#,
        )
        .await;
        assert_eq!(status, 409, "{body}");
        let (status, _) = call_json(
            addr,
            "POST",
            "/api/projects",
            r#"{"name":"Bad","rules":[{"kind":"sender_address","value":"no-at"}]}"#,
        )
        .await;
        assert_eq!(status, 422);

        let path = format!("/api/projects/{id}");
        let (status, body) = send(
            addr,
            "PUT",
            &path,
            HOST,
            Some(r#"{"name":"Store","rules":[]}"#),
        )
        .await;
        assert_eq!(status, 200, "{body}");
        let (_, body) = call(addr, "GET", "/api/messages/1").await;
        assert_eq!(parse(&body)["project"], serde_json::Value::Null);

        assert_eq!(send(addr, "DELETE", &path, HOST, None).await.0, 204);
        assert_eq!(send(addr, "DELETE", &path, HOST, None).await.0, 404);

        // Mail from an unknown SMTP user creates its project on the fly.
        services
            .messages
            .receive(
                Envelope {
                    mail_from: "a@x.example".into(),
                    rcpt_to: vec!["b@x.example".into()],
                    auth_user: AuthUser::parse("blog"),
                },
                b"Subject: Post\r\n\r\nHi\r\n".to_vec(),
            )
            .await
            .unwrap();
        let (_, body) = call(addr, "GET", "/api/projects").await;
        let overview = parse(&body);
        assert_eq!(overview["projects"][0]["name"], "blog");
        assert_eq!(overview["projects"][0]["rules"][0]["kind"], "auth_user");
        assert_eq!(overview["projects"][0]["notify"], true);
    }

    #[tokio::test]
    async fn reads_and_validates_settings() {
        let (addr, _) = serve().await;
        let (_, body) = call(addr, "GET", "/api/settings").await;
        assert_eq!(
            body,
            r#"{"trashRetentionDays":null,"trayGrouping":"all","theme":"system","sidebarCollapsed":false,"notifications":"all","notificationSound":false,"unreadBadge":true,"updateCheck":false}"#
        );

        let (status, body) = call_json(
            addr,
            "PUT",
            "/api/settings",
            r#"{"trashRetentionDays":30,"trayGrouping":"project","theme":"dark","sidebarCollapsed":true,"notifications":"projects","notificationSound":true,"unreadBadge":false,"updateCheck":true}"#,
        )
        .await;
        assert_eq!(
            (status, body.as_str()),
            (
                200,
                r#"{"trashRetentionDays":30,"trayGrouping":"project","theme":"dark","sidebarCollapsed":true,"notifications":"projects","notificationSound":true,"unreadBadge":false,"updateCheck":true}"#
            )
        );
        let (status, _) = call_json(
            addr,
            "PUT",
            "/api/settings",
            r#"{"trashRetentionDays":0,"trayGrouping":"all","theme":"system","sidebarCollapsed":false,"notifications":"all","notificationSound":false,"unreadBadge":true,"updateCheck":false}"#,
        )
        .await;
        assert_eq!(status, 422);
    }

    #[tokio::test]
    async fn marks_all_read_and_reports_stats() {
        let (addr, services) = serve().await;
        let (status, body) = call_json(
            addr,
            "POST",
            "/api/messages/mark-read",
            r#"{"folder":"inbox"}"#,
        )
        .await;
        assert_eq!((status, body.as_str()), (200, r#"{"count":1}"#));
        assert_eq!(call(addr, "POST", "/api/messages/1/unread").await.0, 204);
        assert_eq!(
            services
                .messages
                .folder_counts()
                .await
                .unwrap()
                .inbox
                .unread,
            1
        );
        assert_eq!(call(addr, "POST", "/api/messages/9/unread").await.0, 404);
        services.messages.mark_read(MessageId(1)).await.unwrap();
        assert_eq!(
            services
                .messages
                .folder_counts()
                .await
                .unwrap()
                .inbox
                .unread,
            0
        );

        let (status, body) = call(addr, "GET", "/api/stats").await;
        assert_eq!(status, 200, "{body}");
        assert!(parse(&body)["databaseBytes"].as_u64().unwrap() > 0);

        let (_, body) = call(addr, "GET", "/api/messages?since=9999999999999").await;
        assert_eq!(parse(&body)["total"], 0);
    }

    #[tokio::test]
    async fn resets_all_data() {
        let (addr, services) = serve().await;
        assert_eq!(call(addr, "DELETE", "/api/data").await.0, 204);
        assert_eq!(
            services
                .messages
                .list(ListQuery::default())
                .await
                .unwrap()
                .total,
            0
        );
    }

    #[tokio::test]
    async fn serves_an_openapi_document_of_every_endpoint() {
        let (addr, _) = serve().await;
        let (status, body) = call(addr, "GET", "/api/openapi.json").await;
        assert_eq!(status, 200);
        let spec = parse(&body);
        assert_eq!(spec["info"]["title"], "Mailbeaver API");
        let paths = spec["paths"].as_object().unwrap();
        for path in [
            "/api/messages",
            "/api/messages/{id}",
            "/api/messages/{id}/move",
            "/api/messages/mark-read",
            "/api/projects/{id}",
            "/api/settings",
            "/api/links/check",
            "/api/stats",
            "/api/data",
            "/api/health",
            "/api/logs",
            "/api/v1/messages",
            "/api/v1/message/{id}",
            "/api/v1/search",
        ] {
            assert!(paths.contains_key(path), "{path} is missing");
        }
        assert!(spec["components"]["schemas"]["MessageDetail"].is_object());
    }

    /// Sends one MCP JSON-RPC request with the headers streamable HTTP requires.
    async fn mcp(addr: SocketAddr, origin: Option<&str>) -> (u16, String) {
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}"#;
        let origin = origin
            .map(|o| format!("Origin: {o}\r\n"))
            .unwrap_or_default();
        let request = format!(
            "POST /mcp HTTP/1.1\r\nHost: {HOST}\r\nConnection: close\r\n\
             Content-Type: application/json\r\nAccept: application/json, text/event-stream\r\n\
             {origin}Content-Length: {}\r\n\r\n{body}",
            body.len()
        );
        let mut stream = TcpStream::connect(addr).await.unwrap();
        stream.write_all(request.as_bytes()).await.unwrap();
        let mut response = Vec::new();
        // The SSE stream may stay open; the first chunk holds the answer.
        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            stream.read_to_end(&mut response),
        )
        .await;
        let response = String::from_utf8_lossy(&response).into_owned();
        (response[9..12].parse().unwrap(), response)
    }

    #[tokio::test]
    async fn serves_mcp_to_agents_but_not_to_web_pages() {
        let (addr, _) = serve().await;
        let (status, body) = mcp(addr, None).await;
        assert_eq!(status, 200, "{body}");
        assert!(body.contains("mailbeaver"), "{body}");

        let (status, _) = mcp(addr, Some("https://evil.example")).await;
        assert_eq!(status, 403);
    }

    #[tokio::test]
    async fn remembers_folder_views_and_forgets_deleted_projects() {
        let (addr, _) = serve().await;
        let (_, body) = call(addr, "GET", "/api/folder-views").await;
        assert_eq!(parse(&body)["trash"]["grouping"], "all");
        assert_eq!(parse(&body)["trash"]["pageSize"], 50);

        let (_, body) = call_json(
            addr,
            "POST",
            "/api/projects",
            r#"{"name":"Shop","rules":[]}"#,
        )
        .await;
        let id = parse(&body)["id"].as_i64().unwrap();
        let view =
            format!(r#"{{"grouping":"sender","project":"{id}","period":"7d","pageSize":25}}"#);
        let (status, body) = call_json(addr, "PUT", "/api/folder-views/inbox", &view).await;
        assert_eq!(status, 200, "{body}");
        let (_, body) = call(addr, "GET", "/api/folder-views").await;
        let inbox = &parse(&body)["inbox"];
        assert_eq!(inbox["grouping"], "sender");
        assert_eq!(inbox["project"], id.to_string());
        assert_eq!(inbox["pageSize"], 25);

        let (status, _) = call_json(addr, "PUT", "/api/folder-views/spam", &view).await;
        assert_eq!(status, 400);
        let (status, _) = call_json(
            addr,
            "PUT",
            "/api/folder-views/inbox",
            r#"{"grouping":"sender","project":null,"period":null,"pageSize":40}"#,
        )
        .await;
        assert_eq!(status, 422);

        let path = format!("/api/projects/{id}");
        assert_eq!(send(addr, "DELETE", &path, HOST, None).await.0, 204);
        let (_, body) = call(addr, "GET", "/api/folder-views").await;
        assert_eq!(parse(&body)["inbox"]["project"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn reports_server_info() {
        let (addr, _) = serve().await;
        let (_, body) = call(addr, "GET", "/api/info").await;
        let info = parse(&body);
        assert_eq!(info["smtpAddress"], "127.0.0.1:1026");
        assert_eq!(info["maxMessages"], 10);
        assert_eq!(info["version"], "1.2.3");
    }

    #[tokio::test]
    async fn saves_ports_and_limits_for_the_next_start() {
        let (addr, _) = serve().await;
        let (_, body) = call(addr, "GET", "/api/server-settings").await;
        assert_eq!(
            body,
            r#"{"smtpPort":null,"httpPort":null,"maxMessages":null,"maxMessageMb":null}"#
        );
        let saved = r#"{"smtpPort":2525,"httpPort":null,"maxMessages":100,"maxMessageMb":10}"#;
        let (status, body) = call_json(addr, "PUT", "/api/server-settings", saved).await;
        assert_eq!((status, body.as_str()), (200, saved));
        let (status, _) = call_json(
            addr,
            "PUT",
            "/api/server-settings",
            r#"{"smtpPort":2525,"httpPort":2525,"maxMessages":null,"maxMessageMb":null}"#,
        )
        .await;
        assert_eq!(status, 422);
        let (_, body) = call(addr, "GET", "/api/info").await;
        assert_eq!(parse(&body)["fromEnvironment"][0], "httpPort");
    }

    #[tokio::test]
    async fn suggests_rule_values_from_captured_mail() {
        let (addr, _) = serve().await;
        let suggestions = |path: &'static str| async move { call(addr, "GET", path).await.1 };
        assert_eq!(
            suggestions("/api/projects/rule-suggestions?kind=sender_domain").await,
            r#"[{"value":"shop.example","count":1}]"#
        );
        assert_eq!(
            suggestions("/api/projects/rule-suggestions?kind=sender_address&q=ORD").await,
            r#"[{"value":"orders@shop.example","count":1}]"#
        );
        assert_eq!(
            suggestions("/api/projects/rule-suggestions?kind=sender_domain&q=%25").await,
            "[]"
        );
        assert_eq!(
            suggestions("/api/projects/rule-suggestions?kind=auth_user").await,
            "[]"
        );

        let (status, _) = call_json(
            addr,
            "POST",
            "/api/projects",
            r#"{"name":"Shop","rules":[{"kind":"sender_domain","value":"shop.example"}]}"#,
        )
        .await;
        assert_eq!(status, 201);
        assert_eq!(
            suggestions("/api/projects/rule-suggestions?kind=sender_domain").await,
            "[]",
            "values with a rule are not suggested again"
        );
    }

    #[tokio::test]
    async fn stores_lists_and_clears_ui_log_entries() {
        let (addr, _) = serve().await;
        let (status, _) = call_json(
            addr,
            "POST",
            "/api/logs",
            r#"{"level":"error","target":"query","message":"Request failed"}"#,
        )
        .await;
        assert_eq!(status, 204);
        let (status, _) = call_json(
            addr,
            "POST",
            "/api/logs",
            r#"{"level":"debug","target":"query","message":"x"}"#,
        )
        .await;
        assert_eq!(status, 422);

        let (_, body) = call(addr, "GET", "/api/logs?minLevel=warn&source=frontend").await;
        let entries = parse(&body);
        assert_eq!(entries[0]["message"], "Request failed");
        assert_eq!(entries[0]["source"], "frontend");
        let (_, body) = call(addr, "GET", "/api/logs?source=backend").await;
        assert_eq!(parse(&body).as_array().map(Vec::len), Some(0));

        let (status, _) = call(addr, "DELETE", "/api/logs").await;
        assert_eq!(status, 204);
        let (_, body) = call(addr, "GET", "/api/logs").await;
        assert_eq!(body, "[]");
    }

    #[tokio::test]
    async fn speaks_the_mailpit_api() {
        let (addr, _) = serve().await;
        let (status, body) = call(addr, "GET", "/api/v1/messages").await;
        assert_eq!(status, 200);
        let page = parse(&body);
        assert_eq!(
            (page["total"].as_i64(), page["unread"].as_i64()),
            (Some(1), Some(1))
        );
        assert_eq!(page["messages"][0]["ID"], "1");
        assert_eq!(page["messages"][0]["From"]["Name"], "Shop");
        assert_eq!(
            page["messages"][0]["From"]["Address"],
            "orders@shop.example"
        );

        let (_, body) = call(addr, "GET", "/api/v1/search?query=subject:test").await;
        assert_eq!(parse(&body)["messages_count"], 1);
        let (status, _) = call(addr, "GET", "/api/v1/search?query=").await;
        assert_eq!(status, 400);

        let (_, body) = call(addr, "GET", "/api/v1/message/latest").await;
        let message = parse(&body);
        assert_eq!(message["Subject"], "Test");
        assert_eq!(message["ReturnPath"], "orders@shop.example");
        let (_, body) = call(addr, "GET", "/api/v1/info").await;
        assert_eq!(parse(&body)["Unread"], 0, "reading marks the message read");

        let (_, body) = call(addr, "GET", "/api/v1/message/1/headers").await;
        assert_eq!(parse(&body)["Subject"][0], "Test");
        let (status, body) = call(addr, "GET", "/api/v1/message/1/raw").await;
        assert_eq!(status, 200);
        assert!(body.contains("Subject: Test"));

        let (status, _) = call_json(
            addr,
            "PUT",
            "/api/v1/messages",
            r#"{"IDs":["1"],"Read":false}"#,
        )
        .await;
        assert_eq!(status, 200);
        let (_, body) = call(addr, "GET", "/api/v1/info").await;
        assert_eq!(parse(&body)["Unread"], 1);

        let (status, _) = call_json(addr, "DELETE", "/api/v1/messages", r#"{"IDs":["1"]}"#).await;
        assert_eq!(status, 200);
        let (status, _) = call(addr, "GET", "/api/v1/message/1").await;
        assert_eq!(status, 404);
        let (_, body) = call(addr, "GET", "/api/v1/messages").await;
        assert_eq!(parse(&body)["total"], 0);
    }

    #[tokio::test]
    async fn reports_health_and_serves_the_ui_behind_the_host_check() {
        let (addr, _) = serve().await;
        let (status, body) = call(addr, "GET", "/api/health").await;
        assert_eq!((status, body.as_str()), (200, r#"{"status":"ok"}"#));

        let (status, body) = call(addr, "GET", "/").await;
        assert_eq!((status, body.as_str()), (200, "ui"));
        let (status, _) = send(addr, "GET", "/", "evil.example:8026", None).await;
        assert_eq!(status, 421);
    }
}
