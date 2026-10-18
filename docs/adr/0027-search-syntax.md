# 27. Search syntax with filters

Date: 2026-10-17

## Status

Accepted

## Context

The search field (ADR-0009) matched words only. Developers look for "the unread mail
from the shop since this morning" and agents using the API or MCP want the same
precision without extra parameters. Mailpit, the tool Mailbeaver is compared with, uses
`key:value` filters in its search.

## Decision

The search input understands filters next to free words; everything must match:

| Filter | Meaning |
|---|---|
| `from:`, `to:`, `subject:`, `body:` | Prefix phrase in that column (FTS5 column filter) |
| `is:unread`, `is:read` | Read state |
| `has:attachment` | With attachments |
| `project:<name>`, `project:none` | Project by name (case-insensitive) or none |
| `after:YYYY-MM-DD`, `before:YYYY-MM-DD` | Received on/after, before the UTC day |

- Values with spaces are quoted: `subject:"order 12"`.
- The parser lives in the core (`messages/search.rs`, pure) and is used by
  `GET /api/messages?q=`, the MCP tool `list_messages` and the UI alike.
- The FTS5 expression is built from quoted phrases only; input never becomes FTS5
  syntax. An unknown or invalid filter (`is:starred`, `after:2026-02-30`) stays free
  text instead of being dropped.
- The UI keeps its project and time filters as the clickable alternative and explains
  the syntax in a help popover next to the search field.

## Consequences

- Days are UTC: the server does not know the viewer's time zone. Close to midnight
  `after:` can be off by the local offset.
- No negation and no OR yet; both fit the parser if needed.
- A Mailpit-compatible API can reuse the parser for its `query` parameter.
