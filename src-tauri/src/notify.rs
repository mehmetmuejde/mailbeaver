//! System notifications for new mail and the unread badge on the app icon (ADR-0026).
//! What a notice says is decided in `mailbeaver_core::notifications`; this module only shows it.

use std::collections::BTreeSet;
use std::time::Duration;

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_notification::NotificationExt;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::timeout;
use tracing::{info, warn};

use mailbeaver_core::changes::Change;
use mailbeaver_core::http::Services;
use mailbeaver_core::messages::{MessageId, MessageSummary};
use mailbeaver_core::notifications::{Notice, notice_for, wants_notice};
use mailbeaver_core::settings::Notifications;

use crate::tray::{MAIN_WINDOW, open_message, show_main_window};

/// Mail arriving within this time after the first message shares one notice.
const BURST_WINDOW: Duration = Duration::from_secs(1);
/// The action id the plugin reports for a click on the notification itself.
const TAP: &str = "tap";
/// Notice id that opens the inbox instead of one message.
const INBOX_NOTICE_ID: i32 = 0;

#[cfg(target_os = "macos")]
const SOUND: &str = "Glass";
#[cfg(windows)]
const SOUND: &str = "Mail";
#[cfg(not(any(target_os = "macos", windows)))]
const SOUND: &str = "message-new-email";

/// A click on a notice opens its message, or the inbox for a summary.
pub fn handle_clicks<R: Runtime>(app: &AppHandle<R>) {
    let handle = app.clone();
    let registered = app.notification().on_action(move |performed| {
        if performed.action_id() != TAP {
            return;
        }
        let id = performed
            .notification()
            .map_or(INBOX_NOTICE_ID, |notification| notification.id());
        if id == INBOX_NOTICE_ID {
            show_main_window(&handle);
        } else {
            open_message(&handle, MessageId(i64::from(id)));
        }
    });
    if let Err(err) = registered {
        warn!(error = %err, "notification clicks are not handled");
    }
}

/// Notifies about new mail and keeps the badge current for the lifetime of the app.
pub fn watch<R: Runtime>(app: AppHandle<R>, services: Services) {
    tauri::async_runtime::spawn(async move {
        let mut changes = services.changes.subscribe();
        update_badge(&app, &services).await;
        loop {
            let first = match changes.recv().await {
                Ok(Change::Received { id }) => id,
                Ok(_) | Err(RecvError::Lagged(_)) => {
                    update_badge(&app, &services).await;
                    continue;
                }
                Err(RecvError::Closed) => return,
            };
            let mut received = vec![first];
            // Collects the rest of a burst, e.g. a test suite sending twenty mails.
            while let Ok(next) = timeout(BURST_WINDOW, changes.recv()).await {
                match next {
                    Ok(Change::Received { id }) => received.push(id),
                    Ok(_) | Err(RecvError::Lagged(_)) => {}
                    Err(RecvError::Closed) => return,
                }
            }
            update_badge(&app, &services).await;
            if window_in_front(&app) {
                info!(
                    messages = received.len(),
                    "no notification: the window is in front"
                );
            } else if let Err(err) = notify(&app, &services, &received).await {
                warn!(error = %err, "could not notify about new mail");
            }
        }
    });
}

/// Shows a notice right away, so the settings can show what a notification looks like.
pub fn show_test_notice<R: Runtime>(app: &AppHandle<R>, sound: bool) -> anyhow::Result<()> {
    show(
        app,
        &Notice {
            title: crate::tray::app_name().into(),
            body: "Notifications work. New mail shows up like this.".into(),
            message: None,
        },
        sound,
    )
}

async fn notify<R: Runtime>(
    app: &AppHandle<R>,
    services: &Services,
    received: &[MessageId],
) -> anyhow::Result<()> {
    let settings = services.settings.get().await?;
    if settings.notifications == Notifications::Off {
        info!("no notification: switched off in the settings");
        return Ok(());
    }
    let notifying_projects: BTreeSet<_> = services
        .projects
        .overview()
        .await?
        .projects
        .into_iter()
        .filter(|entry| entry.project.notify)
        .map(|entry| entry.project.id)
        .collect();
    let mut messages: Vec<MessageSummary> = Vec::with_capacity(received.len());
    for id in received {
        // A message can be gone already, e.g. dropped by the message limit.
        if let Some(detail) = services.messages.detail(*id).await? {
            messages.push(detail.summary);
        }
    }
    messages.retain(|message| wants_notice(settings.notifications, message, &notifying_projects));
    match notice_for(&messages) {
        Some(notice) => show(app, &notice, settings.notification_sound),
        None => {
            info!("no notification: no new mail of a project with notifications");
            Ok(())
        }
    }
}

fn show<R: Runtime>(app: &AppHandle<R>, notice: &Notice, sound: bool) -> anyhow::Result<()> {
    let id = notice
        .message
        .and_then(|message| i32::try_from(message.0).ok())
        .unwrap_or(INBOX_NOTICE_ID);
    let mut builder = app
        .notification()
        .builder()
        .id(id)
        .title(&notice.title)
        .body(&notice.body);
    if sound {
        builder = builder.sound(SOUND);
    }
    builder.show()?;
    // In `tauri dev` macOS attributes notices to Terminal and drops them silently when
    // Terminal may not notify; the bundled app asks for permission on first use.
    info!(
        title = %notice.title,
        dev = tauri::is_dev(),
        "notification handed to the system"
    );
    Ok(())
}

/// No notice while the user looks at the app anyway.
fn window_in_front<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.get_webview_window(MAIN_WINDOW).is_some_and(|window| {
        window.is_visible().unwrap_or(false) && window.is_focused().unwrap_or(false)
    })
}

async fn update_badge<R: Runtime>(app: &AppHandle<R>, services: &Services) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        return;
    };
    let count = match badge_count(services).await {
        Ok(count) => count,
        Err(err) => {
            warn!(error = %err, "could not count unread mail for the badge");
            return;
        }
    };
    if let Err(err) = window.set_badge_count(count) {
        warn!(error = %err, "could not set the badge");
    }
}

/// `None` removes the badge: switched off, or nothing unread.
async fn badge_count(services: &Services) -> anyhow::Result<Option<i64>> {
    if !services.settings.get().await?.unread_badge {
        return Ok(None);
    }
    let unread = services.messages.folder_counts().await?.inbox.unread;
    Ok((unread > 0).then_some(unread))
}
