-- Opt-in check for new releases on GitHub (ADR-0032); off until the user turns it on.
ALTER TABLE settings ADD COLUMN update_check INTEGER NOT NULL DEFAULT 0
    CHECK (update_check IN (0, 1));
