# 7. TanStack Router for views and search state

Date: 2026-10-17

## Status

Accepted

## Context

The UI has several views (Inbox, Archive, Trash, Settings) and state that should
survive a reload and be linkable: the search query, the grouping, the project filter
and the open message.

## Decision

TanStack Router manages the routes `/inbox`, `/archive`, `/trash` and `/settings`. View
state lives in typed, validated search params (`q`, `view`, `mailbox`, `message`).
Routes are defined in code (no file-based route generation). The router uses hash
history (`#/inbox`): the Tauri webview and a future static file server then need no
fallback to `index.html` for deep links.

## Consequences

- Reload, back and forward restore the exact view.
- Search params are validated with zod at the edge, like API responses.
- One more runtime dependency (`@tanstack/react-router`).
- URLs contain a `#`; switching to browser history later needs a server-side fallback.
