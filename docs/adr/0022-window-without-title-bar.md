# 22. Window without a native title bar, collapsible sidebar

Date: 2026-10-17

## Status

Superseded by ADR-0023

## Context

The native macOS title bar duplicated the page headers and cost vertical space.

## Decision

On macOS the window uses Tauri's overlay title bar (`titleBarStyle: Overlay`, hidden
title): the native traffic lights sit in the app, the sidebar top and every page header
drag the window (`data-tauri-drag-region`, permission `core:window:allow-start-dragging`).
The sidebar slides in and out with a button next to the logo; while it is hidden, the
button sits in the page header next to the traffic lights. Every page uses the shared
`PageHeader`; Inbox, Archive and Trash put the search on its right. Windows and Linux
keep their native frame.

## Consequences

- Native window behaviour (fullscreen, tiling) stays, as the traffic lights are real.
- Positions were set by numbers (`trafficLightPosition`) and need a visual check.
