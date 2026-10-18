-- The whole schema of Mailbeaver. Later changes go into new numbered files.

-- Projects group messages; rules assign incoming mail to them (ADR-0012).
CREATE TABLE projects (
    id         INTEGER PRIMARY KEY,
    name       TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(name) BETWEEN 1 AND 64),
    -- With notifications set to 'projects', only mail of these projects notifies (ADR-0026).
    notify     INTEGER NOT NULL DEFAULT 1 CHECK (notify IN (0, 1)),
    created_at INTEGER NOT NULL
) STRICT;

-- A sender or SMTP user belongs to at most one project, hence the unique pair.
CREATE TABLE project_rules (
    id         INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    kind       TEXT NOT NULL CHECK (kind IN ('sender_address', 'sender_domain', 'auth_user')),
    value      TEXT NOT NULL CHECK (length(value) BETWEEN 1 AND 254),
    UNIQUE (kind, value)
) STRICT;

CREATE INDEX project_rules_by_project ON project_rules (project_id);

CREATE TABLE messages (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    received_at     INTEGER NOT NULL,
    envelope_from   TEXT NOT NULL,
    envelope_to     TEXT NOT NULL,
    header_from     TEXT NOT NULL,
    header_to       TEXT NOT NULL,
    -- Lowercase address the project rules match against.
    sender_address  TEXT NOT NULL,
    -- SMTP AUTH username as sent; the password is never stored.
    auth_user       TEXT,
    subject         TEXT NOT NULL,
    -- Plain text of the body for search (ADR-0009).
    body_text       TEXT NOT NULL,
    size            INTEGER NOT NULL,
    has_attachments INTEGER NOT NULL CHECK (has_attachments IN (0, 1)),
    is_read         INTEGER NOT NULL DEFAULT 0 CHECK (is_read IN (0, 1)),
    folder          TEXT NOT NULL DEFAULT 'inbox' CHECK (folder IN ('inbox', 'archive', 'trash')),
    trashed_at      INTEGER,
    project_id      INTEGER REFERENCES projects (id) ON DELETE SET NULL,
    raw             BLOB NOT NULL,
    -- Exactly the messages in Trash carry the time they were trashed (ADR-0008).
    CHECK ((folder = 'trash') = (trashed_at IS NOT NULL))
) STRICT;

CREATE INDEX messages_by_folder ON messages (folder, id);
CREATE INDEX messages_by_project ON messages (project_id, id);
CREATE INDEX messages_by_trashed_at ON messages (trashed_at) WHERE trashed_at IS NOT NULL;

-- External-content index: the text lives in `messages`, triggers keep the index in sync.
CREATE VIRTUAL TABLE messages_fts USING fts5 (
    subject, header_from, header_to, body_text,
    content = 'messages', content_rowid = 'id'
);

CREATE TRIGGER messages_fts_insert AFTER INSERT ON messages BEGIN
    INSERT INTO messages_fts (rowid, subject, header_from, header_to, body_text)
    VALUES (new.id, new.subject, new.header_from, new.header_to, new.body_text);
END;

CREATE TRIGGER messages_fts_delete AFTER DELETE ON messages BEGIN
    INSERT INTO messages_fts (messages_fts, rowid, subject, header_from, header_to, body_text)
    VALUES ('delete', old.id, old.subject, old.header_from, old.header_to, old.body_text);
END;

CREATE TRIGGER messages_fts_update
AFTER UPDATE OF subject, header_from, header_to, body_text ON messages BEGIN
    INSERT INTO messages_fts (messages_fts, rowid, subject, header_from, header_to, body_text)
    VALUES ('delete', old.id, old.subject, old.header_from, old.header_to, old.body_text);
    INSERT INTO messages_fts (rowid, subject, header_from, header_to, body_text)
    VALUES (new.id, new.subject, new.header_from, new.header_to, new.body_text);
END;

-- Exactly one row holds the user's settings, including display state that survives
-- restarts (ADR-0024) and the notifications of the desktop app (ADR-0026).
CREATE TABLE settings (
    id                   INTEGER PRIMARY KEY CHECK (id = 1),
    trash_retention_days INTEGER CHECK (trash_retention_days BETWEEN 1 AND 365),
    tray_grouping        TEXT NOT NULL DEFAULT 'all'
        CHECK (tray_grouping IN ('all', 'sender', 'project')),
    theme                TEXT NOT NULL DEFAULT 'system'
        CHECK (theme IN ('system', 'light', 'dark')),
    sidebar_collapsed    INTEGER NOT NULL DEFAULT 0 CHECK (sidebar_collapsed IN (0, 1)),
    notifications        TEXT NOT NULL DEFAULT 'all'
        CHECK (notifications IN ('off', 'all', 'projects')),
    notification_sound   INTEGER NOT NULL DEFAULT 0 CHECK (notification_sound IN (0, 1)),
    unread_badge         INTEGER NOT NULL DEFAULT 1 CHECK (unread_badge IN (0, 1))
) STRICT;
INSERT INTO settings (id) VALUES (1);

-- The last view of each folder; a missing row means the defaults.
CREATE TABLE folder_views (
    folder    TEXT PRIMARY KEY CHECK (folder IN ('inbox', 'archive', 'trash')),
    grouping  TEXT NOT NULL DEFAULT 'all' CHECK (grouping IN ('all', 'sender', 'project')),
    -- 'none' for mail without a project, otherwise a project id; NULL for all projects.
    project   TEXT CHECK (project = 'none' OR CAST(project AS INTEGER) > 0),
    period    TEXT CHECK (period IN ('today', '7d', '30d')),
    page_size INTEGER NOT NULL DEFAULT 50 CHECK (page_size IN (25, 50, 100))
) STRICT;

-- A deleted project must not stay selected as a filter.
CREATE TRIGGER folder_views_forget_project AFTER DELETE ON projects BEGIN
    UPDATE folder_views SET project = NULL WHERE project = CAST(old.id AS TEXT);
END;

-- Log entries of the backend and the UI for Settings → Logs (ADR-0030); only the newest
-- are kept. Never the content of a mail or a password.
CREATE TABLE logs (
    id         INTEGER PRIMARY KEY,
    -- Unix milliseconds.
    created_at INTEGER NOT NULL,
    level      TEXT NOT NULL CHECK (level IN ('error', 'warn', 'info')),
    source     TEXT NOT NULL CHECK (source IN ('backend', 'frontend')),
    -- Rust module or UI area the entry comes from.
    target     TEXT NOT NULL CHECK (length(target) <= 200),
    message    TEXT NOT NULL CHECK (length(message) <= 4000)
) STRICT;
