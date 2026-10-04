-- Add sync-tracking columns to todos.
-- updated_at: RFC3339 timestamp updated on every write; used for LWW conflict resolution.
-- deleted:    soft-delete flag; rows are never physically removed when sync is enabled.
-- sync_id:    stable UUID shared across devices; NULL for todos created before sync was added
--             (will be generated lazily on first sync export).
-- updated_at defaults to '' for pre-existing rows; new rows always get the current
-- timestamp set explicitly by the service layer, so the constant default is safe.
ALTER TABLE todos ADD COLUMN updated_at TEXT NOT NULL DEFAULT '';
ALTER TABLE todos ADD COLUMN deleted    INTEGER NOT NULL DEFAULT 0;
ALTER TABLE todos ADD COLUMN sync_id    TEXT;

-- Device identity persisted for the lifetime of this installation.
-- The value is generated as a UUID v4 on first use.
INSERT OR IGNORE INTO settings (key, value) VALUES ('device_id', '');
