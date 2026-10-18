# 2. The UI talks to the core over the HTTP API, not Tauri IPC

Date: 2026-10-17

## Status

Accepted

## Context

The React/TypeScript UI runs in the Tauri webview today and should also run in a plain
browser when the core runs headless (see ADR-0001). Tauri IPC commands only exist inside
the webview.

## Decision

The UI reads and changes data only through the HTTP API of the core. The single Tauri
command `server_status` tells the desktop UI where that API listens; in a browser the UI
uses its own origin.

## Consequences

- The same UI build works in the desktop app and in the browser.
- The API needs CORS for the Tauri origins and a `Host` check against DNS rebinding.
- Live updates use Server-Sent Events (`/api/events`) instead of Tauri events.
