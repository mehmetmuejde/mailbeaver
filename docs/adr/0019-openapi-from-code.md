# 19. OpenAPI document generated from the handlers

Date: 2026-10-17

## Status

Accepted

## Context

Test suites and tools want to read caught mail through the API (Mailpit's main use in
end-to-end tests). A hand-written specification would drift from the code.

## Decision

The OpenAPI document is generated with utoipa 6 and utoipa-axum 0.3: every handler
carries `#[utoipa::path]`, request and response types derive `ToSchema`, and the feature
routers are `OpenApiRouter`s whose specification is merged in `http.rs`. It is served at
`GET /api/openapi.json` (OpenAPI 3.1) and linked in Settings → General. A test asserts
that the main paths and schemas are present.

## Consequences

- The specification changes together with the code; nothing to keep in sync by hand.
- Domain types carry a `ToSchema` derive — a documentation dependency in the domain,
  accepted for the sake of one source of truth.
- Two new dependencies (`utoipa`, `utoipa-axum`).
