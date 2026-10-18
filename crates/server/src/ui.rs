//! Serves the UI built into the binary (see `build.rs`). The UI uses hash routing, so
//! every page is `/` and only the files themselves need an answer.

use axum::Router;
use axum::extract::Path;
use axum::http::header::{
    CACHE_CONTROL, CONTENT_SECURITY_POLICY, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS,
};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;

include!(concat!(env!("OUT_DIR"), "/assets.rs"));

const INDEX: &str = "index.html";
/// Same rules as the desktop app (`tauri.conf.json`); the API is on the same origin.
const CSP: &str = "default-src 'self'; script-src 'self'; connect-src 'self'; \
    img-src 'self' data: blob: http: https:; style-src 'self' 'unsafe-inline'; \
    font-src 'self' data: http: https:; frame-src 'self'; object-src 'none'; \
    base-uri 'none'; form-action 'none'; frame-ancestors 'none'";
const NOT_BUILT: &str = "The UI is not part of this build. Run `pnpm build` before \
    `cargo build -p mailbeaver-server`. The API is available under /api.";

pub fn router() -> Router {
    Router::new()
        .route("/", get(|| async { asset(INDEX) }))
        .route(
            "/{*path}",
            get(|Path(path): Path<String>| async move { asset(&path) }),
        )
}

fn asset(path: &str) -> Response {
    let Some((_, body)) = ASSETS.iter().find(|(name, _)| *name == path) else {
        return if path == INDEX {
            (StatusCode::SERVICE_UNAVAILABLE, NOT_BUILT).into_response()
        } else {
            StatusCode::NOT_FOUND.into_response()
        };
    };
    // Vite puts a content hash into every file name below `assets/`.
    let cache = if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    (
        [
            (CONTENT_TYPE, HeaderValue::from_static(content_type(path))),
            (CACHE_CONTROL, HeaderValue::from_static(cache)),
            (X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")),
            (CONTENT_SECURITY_POLICY, HeaderValue::from_static(CSP)),
        ],
        *body,
    )
        .into_response()
}

fn content_type(path: &str) -> &'static str {
    match path.rsplit_once('.').map(|(_, extension)| extension) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json" | "map") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_the_content_types() {
        assert_eq!(content_type("index.html"), "text/html; charset=utf-8");
        assert_eq!(
            content_type("assets/index-1.js"),
            "text/javascript; charset=utf-8"
        );
        assert_eq!(content_type("assets/logo.png"), "image/png");
        assert_eq!(content_type("LICENSE"), "application/octet-stream");
    }

    #[test]
    fn answers_unknown_files_with_not_found() {
        assert_eq!(asset("missing.js").status(), StatusCode::NOT_FOUND);
    }
}
