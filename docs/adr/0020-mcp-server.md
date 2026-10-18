# 20. MCP server for agents

Date: 2026-10-17

## Status

Accepted

## Context

Coding agents such as Claude Code test applications that send mail: they need to wait
for a message, read it and follow its confirmation link.

## Decision

The API server also serves the Model Context Protocol at `/mcp` (streamable HTTP, rmcp
3.5). Tools: `list_messages`, `get_message`, `get_message_links`, `wait_for_message`
(up to 60 s, optionally filtered by recipient or subject) and `mark_message_read`. No
tool deletes or changes settings. Besides the `Host` check of the API, every request
with an `Origin` header is rejected, which keeps web pages out; agents send none.
Settings → General shows the `claude mcp add` command.

## Consequences

- Agents connect without an extra binary: `claude mcp add --transport http mailtomb
  http://127.0.0.1:8026/mcp`.
- One new dependency (`rmcp`).
