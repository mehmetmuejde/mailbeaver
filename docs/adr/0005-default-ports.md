# 5. Default ports 1026 (SMTP) and 8026 (HTTP)

Date: 2026-10-17

## Status

Accepted

## Context

Many developers already run Mailpit on its defaults 1025/8025. Both tools should be
able to run side by side during the POC.

## Decision

Mailcatcher listens on 1026 for SMTP and 8026 for HTTP by default, bound to
`127.0.0.1`. `MAILCATCHER_SMTP_PORT`, `MAILCATCHER_HTTP_PORT` and `MAILCATCHER_BIND`
override them.

## Consequences

- Applications must be pointed to port 1026 instead of Mailpit's 1025.
- A port conflict is shown in the window instead of crashing the app.
