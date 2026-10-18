# 15. The tray menu shows the newest inbox messages

Date: 2026-10-17

## Status

Accepted

## Context

Most of the time Mailtomb runs hidden; a glance at the newest mail should not need the
window.

## Decision

- The tray menu lists the ten newest inbox messages: subject, sender and a relative age
  (`now`, `5m`, `3h`, `2d`); unread messages carry a blue dot.
- The setting `trayGrouping` (`all`, `sender`, `project`) groups them into submenus.
- Clicking a message marks it read, shows the window and emits `open-message` to the UI,
  which opens the message in the inbox.
- The menu bar shows the unread inbox count; the tray icon is a monochrome template
  image separate from the app icon.
- The menu is rebuilt on every change event.

## Consequences

- Relative ages avoid a time-zone dependency in Rust; they refresh with the next change,
  not by the clock.
- The tray icon file `src-tauri/icons/tray-icon.png` is a placeholder until the logo set
  exists.
