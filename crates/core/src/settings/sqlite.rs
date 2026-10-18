//! Storage of the settings row and the saved folder views.

use rusqlite::{Connection, params};

use crate::db::DbError;
use crate::messages::Folder;

use super::domain::{
    FolderView, FolderViews, Grouping, Notifications, PageSize, Period, ProjectChoice,
    ServerSettings, Settings, Theme, TrashRetentionDays,
};

pub fn get(conn: &Connection) -> Result<Settings, DbError> {
    conn.query_row(
        "SELECT trash_retention_days, tray_grouping, theme, sidebar_collapsed, notifications,
            notification_sound, unread_badge, update_check
         FROM settings WHERE id = 1",
        [],
        |row| {
            Ok(Settings {
                // The CHECK constraints keep every column valid.
                trash_retention_days: row
                    .get::<_, Option<u16>>(0)?
                    .and_then(|days| TrashRetentionDays::try_from(days).ok()),
                tray_grouping: Grouping::from_db(&row.get::<_, String>(1)?),
                theme: Theme::from_db(&row.get::<_, String>(2)?),
                sidebar_collapsed: row.get(3)?,
                notifications: Notifications::from_db(&row.get::<_, String>(4)?),
                notification_sound: row.get(5)?,
                unread_badge: row.get(6)?,
                update_check: row.get(7)?,
            })
        },
    )
    .map_err(DbError::from)
}

pub fn update(conn: &Connection, settings: &Settings) -> Result<(), DbError> {
    conn.execute(
        "UPDATE settings SET trash_retention_days = ?1, tray_grouping = ?2, theme = ?3,
            sidebar_collapsed = ?4, notifications = ?5, notification_sound = ?6,
            unread_badge = ?7, update_check = ?8
         WHERE id = 1",
        params![
            settings.trash_retention_days.map(TrashRetentionDays::get),
            settings.tray_grouping.as_str(),
            settings.theme.as_str(),
            settings.sidebar_collapsed,
            settings.notifications.as_str(),
            settings.notification_sound,
            settings.unread_badge,
            settings.update_check,
        ],
    )?;
    Ok(())
}

pub fn server(conn: &Connection) -> Result<ServerSettings, DbError> {
    conn.query_row(
        "SELECT smtp_port, http_port, max_messages, max_message_mb FROM settings WHERE id = 1",
        [],
        |row| {
            Ok(ServerSettings {
                smtp_port: row.get(0)?,
                http_port: row.get(1)?,
                max_messages: row.get(2)?,
                max_message_mb: row.get(3)?,
            })
        },
    )
    .map_err(DbError::from)
}

pub fn save_server(conn: &Connection, server: &ServerSettings) -> Result<(), DbError> {
    conn.execute(
        "UPDATE settings SET smtp_port = ?1, http_port = ?2, max_messages = ?3,
            max_message_mb = ?4
         WHERE id = 1",
        params![
            server.smtp_port,
            server.http_port,
            server.max_messages,
            server.max_message_mb,
        ],
    )?;
    Ok(())
}

/// Every folder's view; folders without a saved row get the defaults.
pub fn folder_views(conn: &Connection) -> Result<FolderViews, DbError> {
    let mut stmt =
        conn.prepare("SELECT folder, grouping, project, period, page_size FROM folder_views")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, u16>(4)?,
        ))
    })?;
    let mut views = FolderViews::default();
    for row in rows {
        let (folder, grouping, project, period, page_size) = row?;
        let view = FolderView {
            grouping: Grouping::from_db(&grouping),
            project: project.and_then(|project| ProjectChoice::try_from(project).ok()),
            period: period.as_deref().and_then(Period::from_db),
            page_size: PageSize::try_from(page_size).unwrap_or_default(),
        };
        match folder.parse::<Folder>() {
            Ok(Folder::Inbox) => views.inbox = view,
            Ok(Folder::Archive) => views.archive = view,
            Ok(Folder::Trash) => views.trash = view,
            // The CHECK constraint allows only known folders.
            Err(_) => {}
        }
    }
    Ok(views)
}

pub fn save_folder_view(
    conn: &Connection,
    folder: Folder,
    view: &FolderView,
) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO folder_views (folder, grouping, project, period, page_size)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (folder) DO UPDATE SET grouping = excluded.grouping,
            project = excluded.project, period = excluded.period,
            page_size = excluded.page_size",
        params![
            folder.as_str(),
            view.grouping.as_str(),
            view.project.as_ref().map(ProjectChoice::as_str),
            view.period.map(Period::as_str),
            view.page_size.get(),
        ],
    )?;
    Ok(())
}
