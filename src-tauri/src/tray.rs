//! Tray icon: keeps the app reachable while the window is hidden, shows the unread
//! inbox count and the newest inbox messages; a click opens the message.

use serde::Serialize;
use tauri::image::Image;
use tauri::menu::{IconMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tokio::sync::broadcast::error::RecvError;
use tracing::{info, warn};

use mailbeaver_core::clock::now_millis;
use mailbeaver_core::http::Services;
use mailbeaver_core::messages::{Folder, ListQuery, MessageId};

use crate::tray_menu::{self, TrayActions, TrayMessage, TrayNode};

pub const MAIN_WINDOW: &str = "main";

/// Development builds say so everywhere the user sees the name.
pub fn app_name() -> &'static str {
    if tauri::is_dev() {
        "Mailbeaver (Dev)"
    } else {
        "Mailbeaver"
    }
}
pub const TRAY_ID: &str = "main";
/// Event the UI listens to, to show a message picked in the tray.
const OPEN_MESSAGE_EVENT: &str = "open-message";

const MENU_OPEN: &str = "open";
const MENU_QUIT: &str = "quit";
const MENU_MARK_ALL_READ: &str = "mark-all-read";
const MENU_EMPTY_TRASH: &str = "empty-trash";
const MENU_CAPTURE: &str = "capture";
const MESSAGE_ITEM_PREFIX: &str = "message:";
const TRAY_MESSAGE_LIMIT: u32 = 10;

/// Monochrome template image; macOS tints it for light and dark menu bars.
const TRAY_ICON: &[u8] = include_bytes!("../icons/tray-icon.png");
const DOT_SIZE: u32 = 32;
const UNREAD_DOT_RGB: [u8; 3] = [0x3b, 0x82, 0xf6];

#[derive(Clone, Serialize)]
struct OpenMessage {
    id: MessageId,
}

/// `status_label` is shown as a disabled info line, e.g. the SMTP address or the
/// startup error.
pub fn install(app: &tauri::App, status_label: &str) -> tauri::Result<TrayIcon> {
    let menu = base_menu(app.handle(), status_label, &[], None)?;
    TrayIconBuilder::with_id(TRAY_ID)
        .tooltip(app_name())
        .icon(Image::from_bytes(TRAY_ICON)?)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| on_menu_event(app, event.id().as_ref()))
        .build(app)
}

fn on_menu_event<R: Runtime>(app: &AppHandle<R>, id: &str) {
    match id {
        MENU_OPEN => show_main_window(app),
        MENU_QUIT => app.exit(0),
        MENU_MARK_ALL_READ | MENU_EMPTY_TRASH | MENU_CAPTURE => run_action(app, id),
        _ => {
            if let Some(id) = id
                .strip_prefix(MESSAGE_ITEM_PREFIX)
                .and_then(|id| id.parse().ok())
            {
                open_message(app, MessageId(id));
            }
        }
    }
}

/// The inbox and Trash actions; the change feed refreshes the menu afterwards.
fn run_action<R: Runtime>(app: &AppHandle<R>, id: &str) {
    let Some(services) = app
        .try_state::<Services>()
        .map(|state| state.inner().clone())
    else {
        return;
    };
    let id = id.to_owned();
    tauri::async_runtime::spawn(async move {
        let done = match id.as_str() {
            MENU_MARK_ALL_READ => services
                .messages
                .mark_all_read(Some(Folder::Inbox))
                .await
                .map(drop),
            MENU_EMPTY_TRASH => services.messages.empty_trash().await.map(drop),
            _ => {
                let paused = !services.capture.is_paused();
                services.capture.set_paused(paused);
                info!(paused, "capture switched from the tray");
                Ok(())
            }
        };
        if let Err(err) = done {
            warn!(error = %err, action = id, "tray action failed");
        }
    });
}

/// Marks the message read, brings the window up and tells the UI to show it.
pub(super) fn open_message<R: Runtime>(app: &AppHandle<R>, id: MessageId) {
    show_main_window(app);
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Some(services) = app.try_state::<Services>()
            && let Err(err) = services.messages.mark_read(id).await
        {
            warn!(error = %err, "could not mark message read");
        }
        if let Err(err) = app.emit_to(MAIN_WINDOW, OPEN_MESSAGE_EVENT, OpenMessage { id }) {
            warn!(error = %err, "could not ask the window to open the message");
        }
    });
}

pub fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        return;
    };
    let shown = window
        .show()
        .and_then(|()| window.unminimize())
        .and_then(|()| window.set_focus());
    if let Err(err) = shown {
        warn!(error = %err, "could not show main window");
    }
}

/// Keeps count and message list up to date for the lifetime of the app.
pub fn watch<R: Runtime>(tray: TrayIcon<R>, services: Services, status_label: String) {
    tauri::async_runtime::spawn(async move {
        let mut changes = services.changes.subscribe();
        loop {
            if let Err(err) = refresh(&tray, &services, &status_label).await {
                warn!(error = %err, "could not update tray");
            }
            match changes.recv().await {
                // After a lag the next refresh reads everything anyway.
                Ok(_) | Err(RecvError::Lagged(_)) => {}
                Err(RecvError::Closed) => return,
            }
        }
    });
}

async fn refresh<R: Runtime>(
    tray: &TrayIcon<R>,
    services: &Services,
    status_label: &str,
) -> anyhow::Result<()> {
    let counts = services.messages.folder_counts().await?;
    let inbox = services
        .messages
        .list(ListQuery {
            folder: Some(Folder::Inbox),
            limit: TRAY_MESSAGE_LIMIT,
            ..ListQuery::default()
        })
        .await?;
    let grouping = services.settings.get().await?.tray_grouping;
    let nodes = tray_menu::layout(&inbox.messages, grouping, now_millis());

    let unread = counts.inbox.unread;
    let header = match (counts.inbox.total, unread) {
        (0, _) => "Inbox is empty".to_owned(),
        (_, 0) => "Inbox".to_owned(),
        (_, unread) => format!("Inbox – {unread} unread"),
    };
    let paused = services.capture.is_paused();
    let actions = tray_menu::actions(unread, counts.trash.total, paused);
    let status = if paused {
        format!("{status_label} (paused)")
    } else {
        status_label.to_owned()
    };
    let menu = base_menu(
        tray.app_handle(),
        &status,
        &[(header, nodes)],
        Some(&actions),
    )?;
    tray.set_menu(Some(menu))?;
    tray.set_title((unread > 0).then(|| unread.to_string()))?;
    tray.set_tooltip(Some(format!("{} – {unread} unread", app_name())))?;
    Ok(())
}

/// Open item, the inbox section (when there is one), the actions, the status line and
/// Quit.
fn base_menu<R: Runtime>(
    app: &AppHandle<R>,
    status_label: &str,
    inbox: &[(String, Vec<TrayNode>)],
    actions: Option<&TrayActions>,
) -> tauri::Result<Menu<R>> {
    let menu = Menu::new(app)?;
    menu.append(&MenuItem::with_id(
        app,
        MENU_OPEN,
        format!("Open {}", app_name()),
        true,
        None::<&str>,
    )?)?;
    for (header, nodes) in inbox {
        menu.append(&PredefinedMenuItem::separator(app)?)?;
        menu.append(&MenuItem::new(app, header, false, None::<&str>)?)?;
        let dots = Dots::new();
        for node in nodes {
            match node {
                TrayNode::Message(message) => menu.append(&message_item(app, message, &dots)?)?,
                TrayNode::Group { label, messages } => {
                    let unread = messages.iter().filter(|message| message.unread).count();
                    let text = if unread > 0 {
                        format!("{label} – {unread} unread")
                    } else {
                        label.clone()
                    };
                    let submenu = Submenu::new(app, text, true)?;
                    for message in messages {
                        submenu.append(&message_item(app, message, &dots)?)?;
                    }
                    menu.append(&submenu)?;
                }
            }
        }
    }
    if let Some(actions) = actions {
        menu.append(&PredefinedMenuItem::separator(app)?)?;
        let (label, enabled) = &actions.mark_all_read;
        menu.append(&MenuItem::with_id(
            app,
            MENU_MARK_ALL_READ,
            label,
            *enabled,
            None::<&str>,
        )?)?;
        let (label, enabled) = &actions.empty_trash;
        menu.append(&MenuItem::with_id(
            app,
            MENU_EMPTY_TRASH,
            label,
            *enabled,
            None::<&str>,
        )?)?;
        menu.append(&MenuItem::with_id(
            app,
            MENU_CAPTURE,
            &actions.capture,
            true,
            None::<&str>,
        )?)?;
    }
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::new(app, status_label, false, None::<&str>)?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(
        app,
        MENU_QUIT,
        format!("Quit {}", app_name()),
        true,
        None::<&str>,
    )?)?;
    Ok(menu)
}

fn message_item<R: Runtime>(
    app: &AppHandle<R>,
    message: &TrayMessage,
    dots: &Dots,
) -> tauri::Result<impl IsMenuItem<R>> {
    // Read messages get a transparent dot, so all labels stay aligned.
    let icon = if message.unread {
        dots.unread.clone()
    } else {
        dots.read.clone()
    };
    IconMenuItem::with_id(
        app,
        format!("{MESSAGE_ITEM_PREFIX}{}", message.id.0),
        &message.label,
        true,
        Some(icon),
        None::<&str>,
    )
}

/// The blue unread dot and its transparent counterpart, drawn once per refresh.
struct Dots {
    unread: Image<'static>,
    read: Image<'static>,
}

impl Dots {
    fn new() -> Self {
        Self {
            unread: dot(Some(UNREAD_DOT_RGB)),
            read: dot(None),
        }
    }
}

/// A filled circle in the middle third of a square image; `None` stays transparent.
fn dot(color: Option<[u8; 3]>) -> Image<'static> {
    let size = DOT_SIZE;
    let center = f64::from(size) / 2.0;
    let radius = f64::from(size) / 6.0;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let dx = f64::from(x) + 0.5 - center;
            let dy = f64::from(y) + 0.5 - center;
            let inside = (dx * dx + dy * dy).sqrt() <= radius;
            match color {
                Some([r, g, b]) if inside => rgba.extend_from_slice(&[r, g, b, 0xff]),
                _ => rgba.extend_from_slice(&[0, 0, 0, 0]),
            }
        }
    }
    Image::new_owned(rgba, size, size)
}
