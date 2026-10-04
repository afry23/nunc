-- Persist the originating device for synced sessions so they are not re-exported
-- as if they had been created on the current device.
ALTER TABLE sessions ADD COLUMN sync_device_id TEXT;

-- Existing local sessions belong to this installation's device.
UPDATE sessions
SET sync_device_id = (SELECT value FROM settings WHERE key = 'device_id')
WHERE sync_device_id IS NULL OR sync_device_id = '';
