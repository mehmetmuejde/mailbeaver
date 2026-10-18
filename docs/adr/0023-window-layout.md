# 23. Window layout: no native title bar, sidebar toggle in the page header

Date: 2026-10-17

## Status

Accepted. Supersedes ADR-0022.

## Context

ADR-0022 put the sidebar toggle next to the logo; in use it felt out of place, and the
logo looked better centered.

## Decision

- On macOS the window uses Tauri's overlay title bar (`titleBarStyle: Overlay`, hidden
  title): the native traffic lights sit in the app; the sidebar top and every page
  header drag the window (`data-tauri-drag-region`, permission
  `core:window:allow-start-dragging`). Windows and Linux keep their native frame.
- The sidebar shows the logo and name centered; it slides in and out with a toggle at
  the start of every page header (shared `PageHeader`). While the sidebar is hidden, the
  header leaves room for the traffic lights.
- Inbox, Archive and Trash put the search at the right of their header.

## Consequences

- Native window behaviour (fullscreen, tiling) stays, as the traffic lights are real.
- Positions were set by numbers (`trafficLightPosition`) and need a visual check.
