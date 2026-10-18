# 21. Link check on demand

Date: 2026-10-17

## Status

Accepted

## Context

Mail under test often contains confirmation or reset links; whether they work is part
of the test. Browsers cannot read the status of cross-origin requests.

## Decision

`POST /api/links/check` requests up to 50 http(s) URLs with reqwest (rustls): HEAD with a
GET fallback for 405/501, 5 s timeout, up to 5 redirects, no cookies, eight at a time.
It runs only when the user presses "Check links" in the Links tab. The JSON body forces
a CORS preflight, so foreign pages cannot use Mailtomb to send requests.

## Consequences

- The server makes outgoing requests on demand, including to local addresses — that is
  the point for links to the app under development.
- reqwest becomes a direct dependency (it was already in the build through Tauri).
