# 24. Display state is stored in the database

Date: 2026-10-17

## Status

Accepted

## Context

Choices on screen should survive a restart: how each folder is grouped and filtered,
whether the sidebar is open, the theme. Browser storage of the webview depends on the
origin (`localhost:1420` in development, `tauri://` in the app), is lost when the
webview data is cleared, and is not shared with a browser UI of a future headless server.

## Decision

- Schema version 2 adds `settings.theme` and `settings.sidebar_collapsed`, and a table
  `folder_views` (grouping, project filter, period, page size per folder). A trigger
  clears a deleted project from saved filters; "Reset Mailtomb" removes the views.
- API: `theme` and `sidebarCollapsed` are part of `GET/PUT /api/settings`;
  `GET /api/folder-views` and `PUT /api/folder-views/{folder}` read and save views.
- Every change of grouping, filters or page size saves the folder's view. Entering a
  folder without view parameters in the URL restores it; changes on the page itself
  are never overwritten.
- Theme and sidebar keep a copy in browser storage only so the first paint is right;
  the database value wins as soon as it is loaded, and later changes are saved there.

## Consequences

- Desktop app, browser and a future Docker UI share the same display state.
- Two windows changing theme or sidebar at the same time: the last change wins.
