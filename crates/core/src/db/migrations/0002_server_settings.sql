-- Ports and limits chosen in the app (ADR-0031); NULL keeps the default. Environment
-- variables still win over these values.
ALTER TABLE settings ADD COLUMN smtp_port INTEGER CHECK (smtp_port BETWEEN 1 AND 65535);
ALTER TABLE settings ADD COLUMN http_port INTEGER CHECK (http_port BETWEEN 1 AND 65535);
ALTER TABLE settings ADD COLUMN max_messages INTEGER
    CHECK (max_messages BETWEEN 1 AND 1000000);
ALTER TABLE settings ADD COLUMN max_message_mb INTEGER CHECK (max_message_mb BETWEEN 1 AND 1024);
