//! Reading and saving the settings; a change is announced so the trash retention and
//! the tray apply it right away.

use crate::changes::{Change, ChangeFeed};
use crate::db::{Database, DbError};

use crate::messages::Folder;

use std::path::Path;

use super::domain::{FolderView, FolderViews, ServerSettings, Settings};
use super::sqlite;

#[derive(Clone)]
pub struct SettingsService {
    db: Database,
    changes: ChangeFeed,
}

impl SettingsService {
    pub fn new(db: Database, changes: ChangeFeed) -> Self {
        Self { db, changes }
    }

    pub async fn get(&self) -> Result<Settings, DbError> {
        self.db.run(|conn| sqlite::get(conn)).await
    }

    pub async fn folder_views(&self) -> Result<FolderViews, DbError> {
        self.db.run(|conn| sqlite::folder_views(conn)).await
    }

    /// Remembers how a folder is shown, so it opens the same way next time.
    pub async fn save_folder_view(
        &self,
        folder: Folder,
        view: FolderView,
    ) -> Result<FolderView, DbError> {
        let saved = view.clone();
        self.db
            .run(move |conn| sqlite::save_folder_view(conn, folder, &saved))
            .await?;
        self.changes.announce(Change::SettingsChanged);
        Ok(view)
    }

    pub async fn server(&self) -> Result<ServerSettings, DbError> {
        self.db.run(|conn| sqlite::server(conn)).await
    }

    /// Validated by the caller; applies at the next start.
    pub async fn save_server(&self, server: ServerSettings) -> Result<ServerSettings, DbError> {
        self.db
            .run(move |conn| sqlite::save_server(conn, &server))
            .await?;
        self.changes.announce(Change::SettingsChanged);
        Ok(server)
    }

    pub async fn update(&self, settings: Settings) -> Result<Settings, DbError> {
        self.db
            .run(move |conn| sqlite::update(conn, &settings))
            .await?;
        self.changes.announce(Change::SettingsChanged);
        Ok(settings)
    }
}

/// The saved ports and limits of a database that no server has open, e.g. after the
/// servers failed to start (ADR-0031).
pub fn read_server_settings_at(database: &Path) -> Result<ServerSettings, DbError> {
    Database::open(database)?.with(|conn| sqlite::server(conn))
}

/// Saves ports and limits into a database that no server has open.
pub fn save_server_settings_at(database: &Path, server: &ServerSettings) -> Result<(), DbError> {
    Database::open(database)?.with(|conn| sqlite::save_server(conn, server))
}
