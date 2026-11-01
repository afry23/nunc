-- Todoist and Google Drive sync were removed (ADR 0024): only WebDAV remains.
-- Deliberately incompatible clean-up: the mapping table and the settings of both providers go away.
DROP TABLE IF EXISTS todoist_mappings;
DELETE FROM settings WHERE key IN ('last_synced_at', 'drive_folder_id', 'todoist.last_sync_at');
UPDATE settings SET value = 'none' WHERE key = 'sync_provider' AND value = 'google_drive';
