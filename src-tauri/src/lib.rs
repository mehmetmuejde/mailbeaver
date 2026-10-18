//! Desktop host of Mailbeaver (ADR-0001): starts the core's servers inside the Tauri
//! process, tells the UI where the HTTP API listens, keeps running in the tray when the
//! window is closed and notifies about new mail. The only crate that uses Tauri.

mod notify;
mod tray;
mod tray_menu;

use serde::Serialize;
use tauri::{Manager, RunEvent, WindowEvent};
use tracing::error;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use mailbeaver_core::config::Config;
use mailbeaver_core::logs::{self, LogReceiver};
use mailbeaver_core::server::{self, Host, HostKind};
use mailbeaver_core::settings::{ServerSettings, read_server_settings_at, save_server_settings_at};

/// What the UI needs to know about the embedded servers.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ServerStatus {
    api_base_url: Option<String>,
    smtp_address: Option<String>,
    error: Option<String>,
}

#[tauri::command]
fn server_status(status: tauri::State<'_, ServerStatus>) -> ServerStatus {
    status.inner().clone()
}

/// Starts the app again, so saved ports and limits apply (ADR-0031).
#[tauri::command]
fn restart_app(app: tauri::AppHandle) {
    app.restart();
}

/// Saves new ports while the servers are down, e.g. after a port conflict; the HTTP
/// API that would normally take them is not running then.
#[tauri::command]
fn save_server_ports(app: tauri::AppHandle, smtp_port: u16, http_port: u16) -> Result<(), String> {
    save_ports(&app, smtp_port, http_port).map_err(|err| format!("{err:#}"))
}

fn save_ports(app: &tauri::AppHandle, smtp_port: u16, http_port: u16) -> anyhow::Result<()> {
    let config = Config::from_env(app.path().app_data_dir()?)?;
    anyhow::ensure!(
        !config.from_env.smtp_port && !config.from_env.http_port,
        "the ports are set by MAILBEAVER_SMTP_PORT or MAILBEAVER_HTTP_PORT; change them there"
    );
    let saved = ServerSettings {
        smtp_port: Some(smtp_port),
        http_port: Some(http_port),
        ..read_server_settings_at(&config.database)?
    };
    saved.validate()?;
    save_server_settings_at(&config.database, &saved)?;
    Ok(())
}

/// Lets the settings show what a notification looks like (and trigger the system's
/// permission prompt).
#[tauri::command]
fn send_test_notification(app: tauri::AppHandle, sound: bool) -> Result<(), String> {
    notify::show_test_notice(&app, sound).map_err(|err| format!("{err:#}"))
}

fn start_servers(app: &tauri::App, logs: Option<LogReceiver>) -> anyhow::Result<server::Running> {
    let data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&data_dir)?;
    let config = Config::from_env(data_dir)?;
    tauri::async_runtime::block_on(server::start(
        config,
        Host {
            kind: HostKind::Desktop,
            ui: None,
            logs,
        },
    ))
}

fn setup(
    app: &mut tauri::App,
    logs: Option<LogReceiver>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (status, services) = match start_servers(app, logs) {
        Ok(running) => (
            ServerStatus {
                api_base_url: Some(format!("http://{}", running.http)),
                smtp_address: Some(running.smtp.to_string()),
                error: None,
            },
            Some(running.services),
        ),
        Err(err) => {
            // The window still opens and shows the error, e.g. a port already in use.
            error!(error = format!("{err:#}"), "mail servers did not start");
            let status = ServerStatus {
                api_base_url: None,
                smtp_address: None,
                error: Some(format!("{err:#}")),
            };
            (status, None)
        }
    };

    let status_label = match &status.smtp_address {
        Some(address) => format!("SMTP: {address}"),
        None => "SMTP: not running".to_owned(),
    };
    match tray::install(app, &status_label) {
        Ok(tray) => {
            if let Some(services) = &services {
                tray::watch(tray, services.clone(), status_label);
            }
        }
        // Without a tray the app still works; closing the window then quits it.
        Err(err) => error!(error = %err, "tray icon could not be created"),
    }

    if let Some(services) = services {
        notify::handle_clicks(app.handle());
        notify::watch(app.handle().clone(), services.clone());
        app.manage(services);
    }
    app.manage(status);
    Ok(())
}

/// Hides the main window instead of closing it while a tray icon can bring it back.
fn on_window_event(window: &tauri::Window, event: &WindowEvent) {
    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };
    let has_tray = window.app_handle().tray_by_id(tray::TRAY_ID).is_some();
    if window.label() == tray::MAIN_WINDOW && has_tray {
        api.prevent_close();
        if let Err(err) = window.hide() {
            error!(error = %err, "could not hide main window");
        }
    }
}

fn on_run_event(app: &tauri::AppHandle, event: RunEvent) {
    // Clicking the dock icon of a hidden app on macOS.
    #[cfg(target_os = "macos")]
    if let RunEvent::Reopen { .. } = event {
        tray::show_main_window(app);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, event);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Console output plus the database for Settings → Logs. A second subscriber (e.g.
    // in tests) is not an error worth stopping for; the log then stays empty.
    let (database_layer, receiver) = logs::database_layer();
    let logs = tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .with(database_layer)
        .try_init()
        .is_ok()
        .then_some(receiver);
    let logs = std::sync::Mutex::new(logs);

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| {
            let logs = logs
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            setup(app, logs)
        })
        .on_window_event(on_window_event)
        .invoke_handler(tauri::generate_handler![
            server_status,
            send_test_notification,
            restart_app,
            save_server_ports
        ])
        .build(tauri::generate_context!());
    match app {
        Ok(app) => app.run(on_run_event),
        Err(err) => {
            error!(error = %err, "tauri application failed to start");
            std::process::exit(1);
        }
    }
}
