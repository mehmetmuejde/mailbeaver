# 26. System notifications and unread badge

Date: 2026-10-17

## Status

Accepted

## Context

Mailbeaver mostly runs hidden in the tray. A developer waiting for a mail from the
application under test wants to know when it arrives without watching the tray. The
core must stay free of Tauri (ADR-0001), and a test suite can send dozens of mails per
second.

## Decision

- The desktop host shows system notifications with `tauri-plugin-notification`
  (`=2.6.0`, Rust API only, no JavaScript permission). Only `desktop/notify.rs` uses it.
- What notifies and what a notice says is pure core code (`notifications.rs`):
  settings `notifications` = `off` | `all` | `projects` (default `all`), where
  `projects` means projects with their `notify` flag on (default on).
- Only inbox mail notifies, and only while the main window is not in front.
- Mail arriving within one second after the first message shares one notice
  ("5 new messages"); a single message shows sender, project, subject and preview.
- The notice id is the message id: a click opens the message, a summary (id 0) opens
  the window. The plugin reports clicks on desktop since 2.6.
- Optional system sound (`notification_sound`, default off).
- The unread inbox count is shown on the app icon (`unread_badge`, default on) with
  Tauri's `set_badge_count` — macOS and Linux; Windows has no badge.
- Settings → Notifications offers a test notification (Tauri command
  `send_test_notification`).

## Consequences

- Migration `0003_notifications.sql` adds the three settings columns and
  `projects.notify`.
- In `pnpm tauri dev` macOS attributes notifications to Terminal; clicking and the app
  name only work in the bundled app.
- On macOS and Windows the plugin waits on one thread per shown notice until it is
  clicked or dismissed; the one-second batching keeps their number low.
- A headless host simply does not notify; the settings stay in the API.
